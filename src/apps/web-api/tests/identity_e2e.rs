//! E2E against the real binary (the M1 exit criterion, ROADMAP L47).
//!
//! Drives the built `medioss` binary via CARGO_BIN_EXE (never invokes
//! cargo), with a temp MEDIOSS_DATA_DIR, fixed SESSION_SECRET, explicit
//! insecure cookie mode for the local HTTP roundtrip, and an RAII guard
//! that kills the child on success or panic. HTTP is spoken with a minimal
//! std-only client so no new test dependency is introduced.

use std::{
    io::{Read, Write},
    process::{Child, Command},
    time::{Duration, Instant},
};

struct ServerGuard {
    child: Child,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct E2eServer {
    _guard: ServerGuard,
    port: u16,
    _data_dir: tempfile::TempDir,
}

impl E2eServer {
    async fn request(
        &self,
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> Response {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", self.port))
            .expect("connect to server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut raw = format!("{method} {path} HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\n");
        for (name, value) in headers {
            raw.push_str(&format!("{name}: {value}\r\n"));
        }
        if let Some(body) = body {
            raw.push_str("content-type: application/json\r\n");
            raw.push_str(&format!("content-length: {}\r\n", body.len()));
            raw.push_str("\r\n");
            raw.push_str(body);
        } else {
            raw.push_str("\r\n");
        }
        stream.write_all(raw.as_bytes()).expect("write request");

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).expect("read response");
        parse_response(&buffer)
    }
}

struct Response {
    status: u16,
    set_cookie: Option<String>,
    body: String,
}

fn parse_response(raw: &[u8]) -> Response {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .unwrap_or((&text, ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let set_cookie = head
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("set-cookie:"))
        .map(|line| line["set-cookie:".len()..].trim().to_owned());
    Response {
        status,
        set_cookie,
        body: body.to_owned(),
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

async fn spawn_server(extra_env: &[(&str, &str)]) -> E2eServer {
    let data_dir = tempfile::tempdir().expect("temp data dir");
    let static_dir = tempfile::tempdir().expect("temp static dir");
    let port = free_port();
    let mut command = Command::new(env!("CARGO_BIN_EXE_medioss"));
    command
        .env("MEDIOSS_DATA_DIR", data_dir.path())
        .env("MEDIOSS_STATIC_DIR", static_dir.path())
        .env("SESSION_SECRET", "e2e-secret-value")
        .env("MEDIOSS_ORIGIN", "http://localhost")
        .env("SESSION_COOKIE_SECURE", "false")
        .env("PORT", port.to_string());
    for (name, value) in extra_env {
        command.env(name, value);
    }
    let child = command.spawn().expect("spawn medioss binary");
    let guard = ServerGuard { child };

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(true) = tokio::task::spawn_blocking(move || health_probe(port)).await {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "server did not become healthy in 15s"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    E2eServer {
        _guard: guard,
        port,
        _data_dir: data_dir,
    }
}

fn health_probe(port: u16) -> bool {
    let mut stream = match std::net::TcpStream::connect(("127.0.0.1", port)) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    if stream
        .write_all(b"GET /api/health HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut buffer = Vec::new();
    if stream.read_to_end(&mut buffer).is_err() {
        return false;
    }
    String::from_utf8_lossy(&buffer).starts_with("HTTP/1.1 200")
}

fn origin_headers() -> Vec<(String, String)> {
    vec![("origin".to_owned(), "http://localhost".to_owned())]
}

fn headers_with_cookie(cookie: &Option<String>) -> Vec<(String, String)> {
    let mut headers = origin_headers();
    if let Some(value) = cookie {
        headers.push((
            "cookie".to_owned(),
            value.split(';').next().unwrap_or("").to_owned(),
        ));
    }
    headers
}

fn headers_origin_secret(secret: &str) -> Vec<(String, String)> {
    vec![
        ("origin".to_owned(), "http://localhost".to_owned()),
        ("x-setup-secret".to_owned(), secret.to_owned()),
    ]
}

#[tokio::test]
async fn e2e_setup_login_me_logout_and_cli_reset() {
    let server = spawn_server(&[]).await;

    // 1. Setup status: required while uninitialized.
    let status = server.request("GET", "/api/identity/setup-status", &[], None).await;
    assert_eq!(status.status, 200);
    assert!(status.body.contains("\"setup_required\":true"));

    // 2. Health stays 200 without a session; /me is 401 JSON.
    let health = server.request("GET", "/api/health", &[], None).await;
    assert_eq!(health.status, 200);
    let me = server.request("GET", "/api/identity/me", &[], None).await;
    assert_eq!(me.status, 401);
    assert!(me.body.contains("invalid credentials"));

    // 3. Setup rejects missing/incorrect possession secret.
    let no_secret = server
        .request(
            "POST",
            "/api/identity/setup",
            &origin_headers(),
            Some(r#"{"email":"admin@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(no_secret.status, 403);
    let wrong_secret = server
        .request(
            "POST",
            "/api/identity/setup",
            &headers_origin_secret("wrong"),
            Some(r#"{"email":"admin@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(wrong_secret.status, 403);

    // 4. Setup rejects a missing Origin (CSRF).
    let no_origin = server
        .request(
            "POST",
            "/api/identity/setup",
            &[("x-setup-secret".to_owned(), "e2e-secret-value".to_owned())],
            Some(r#"{"email":"admin@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(no_origin.status, 403);

    // 5. Correct setup creates the admin.
    let setup = server
        .request(
            "POST",
            "/api/identity/setup",
            &headers_origin_secret("e2e-secret-value"),
            Some(r#"{"email":"admin@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(setup.status, 201, "setup body: {}", setup.body);

    // 6. Second setup attempt fails: exactly-once semantics.
    let second = server
        .request(
            "POST",
            "/api/identity/setup",
            &headers_origin_secret("e2e-secret-value"),
            Some(r#"{"email":"other@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(second.status, 409);

    // 7. No self-registration endpoints exist.
    let register = server
        .request("POST", "/api/identity/register", &origin_headers(), Some("{}"))
        .await;
    let signup = server
        .request("POST", "/api/identity/signup", &origin_headers(), Some("{}"))
        .await;
    assert_eq!(register.status, 404);
    assert_eq!(signup.status, 404);

    // 8. Wrong password fails generically.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let bad_login = server
        .request(
            "POST",
            "/api/identity/login",
            &origin_headers(),
            Some(r#"{"email":"admin@example.com","password":"wrong-password"}"#),
        )
        .await;
    assert_eq!(bad_login.status, 401);
    assert!(bad_login.body.contains("invalid credentials"));

    // 9. Correct login issues an opaque cookie.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let login = server
        .request(
            "POST",
            "/api/identity/login",
            &origin_headers(),
            Some(r#"{"email":"admin@example.com","password":"long-enough-pass"}"#),
        )
        .await;
    assert_eq!(login.status, 200, "login body: {}", login.body);
    let cookie = login.set_cookie.clone().expect("login must set a cookie");
    assert!(cookie.contains("medioss-session"), "cookie: {cookie}");

    // 10. Authenticated /me with the cookie.
    let me_authed = server
        .request("GET", "/api/identity/me", &headers_with_cookie(&Some(cookie.clone())), None)
        .await;
    assert_eq!(me_authed.status, 200);
    assert!(me_authed.body.contains("admin@example.com"));

    // 11. Admin creates a member; the member gets a temporary password.
    let created = server
        .request(
            "POST",
            "/api/identity/users",
            &headers_with_cookie(&Some(cookie.clone())),
            Some(r#"{"email":"member@example.com","password":"temp-pass-12345"}"#),
        )
        .await;
    assert_eq!(created.status, 201, "create body: {}", created.body);
    assert!(created.body.contains("\"must_change_password\":true"));

    // 12. Unauthenticated user creation is rejected.
    let anon_create = server
        .request(
            "POST",
            "/api/identity/users",
            &origin_headers(),
            Some(r#"{"email":"anon@example.com","password":"temp-pass-12345"}"#),
        )
        .await;
    assert_eq!(anon_create.status, 401);

    // 13. Member cannot create users (admin-only).
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let member_login = server
        .request(
            "POST",
            "/api/identity/login",
            &origin_headers(),
            Some(r#"{"email":"member@example.com","password":"temp-pass-12345"}"#),
        )
        .await;
    assert_eq!(member_login.status, 200);
    let member_cookie = member_login.set_cookie.expect("member cookie");
    let member_create = server
        .request(
            "POST",
            "/api/identity/users",
            &headers_with_cookie(&Some(member_cookie.clone())),
            Some(r#"{"email":"sneaky@example.com","password":"temp-pass-12345"}"#),
        )
        .await;
    assert_eq!(member_create.status, 403);

    // 14. Member's forced password change issues a fresh session.
    let changed = server
        .request(
            "POST",
            "/api/identity/me/password",
            &headers_with_cookie(&Some(member_cookie.clone())),
            Some(r#"{"email":"member@example.com","password":"permanent-pass-1"}"#),
        )
        .await;
    assert_eq!(changed.status, 200, "change body: {}", changed.body);
    assert!(changed.body.contains("\"must_change_password\":false"));

    // 15. Logout makes the pre-logout cookie unusable.
    let logout = server
        .request(
            "POST",
            "/api/identity/logout",
            &headers_with_cookie(&Some(cookie.clone())),
            None,
        )
        .await;
    assert_eq!(logout.status, 204);
    let me_after_logout = server
        .request("GET", "/api/identity/me", &headers_with_cookie(&Some(cookie)), None)
        .await;
    assert_eq!(me_after_logout.status, 401);
}

#[tokio::test]
async fn e2e_throttle_blocks_repeated_failures_for_known_and_unknown_users() {
    let server = spawn_server(&[]).await;
    for _ in 0..3 {
        let _ = server
            .request(
                "POST",
                "/api/identity/login",
                &origin_headers(),
                Some(r#"{"email":"who-is-this@example.com","password":"whatever-pass"}"#),
            )
            .await;
    }
    let throttled = server
        .request(
            "POST",
            "/api/identity/login",
            &origin_headers(),
            Some(r#"{"email":"who-is-this@example.com","password":"whatever-pass"}"#),
        )
        .await;
    assert_eq!(throttled.status, 429);
    assert!(throttled.body.contains("too many attempts"));
}
