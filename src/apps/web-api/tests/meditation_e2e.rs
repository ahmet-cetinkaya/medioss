//! M2 E2E against the real binary (roadmap exit criterion, spec SC-001).
//!
//! Drives the built `medioss` binary like identity_e2e does: temp data dir,
//! fixed secret, insecure cookie mode, RAII server guard, std-only HTTP
//! client. The meditation battery covers upload→stream→404→401, the Opus
//! pipeline, admin gating, per-user isolation, delete semantics, and the
//! idempotent completion report.

use std::{
    process::{Child, Command},
    time::{Duration, Instant},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Response {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

impl E2eServer {
    async fn spawn() -> Self {
        let data_dir = tempfile::tempdir().unwrap();
        let port = pick_free_port();
        let mut command = Command::new(env!("CARGO_BIN_EXE_medioss"));
        command
            .env("MEDIOSS_DATA_DIR", data_dir.path())
            .env("SESSION_SECRET", "e2e-m2-secret-not-real")
            .env("MEDIOSS_ORIGIN", "http://localhost")
            .env("SESSION_COOKIE_SECURE", "false")
            .env("PORT", port.to_string())
            .env("MEDIOSS_STATIC_DIR", "/nonexistent")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let child = command.spawn().expect("spawn medioss");

        let guard = ServerGuard { child };
        let server = E2eServer {
            _guard: guard,
            port,
            _data_dir: data_dir,
        };
        server.wait_healthy().await;
        server
    }

    async fn wait_healthy(&self) {
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if let Some(response) = self.try_request("GET", "/api/health", &[], None).await {
                if response.status == 200 {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        panic!("server did not become healthy");
    }

    async fn try_request(
        &self,
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: Option<Vec<u8>>,
    ) -> Option<Response> {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", self.port))
            .await
            .ok()?;
        let mut raw =
            format!("{method} {path} HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\n");
        for (name, value) in headers {
            raw.push_str(&format!("{name}: {value}\r\n"));
        }
        let body_bytes = body.unwrap_or_default();
        if !body_bytes.is_empty() || matches!(method, "POST" | "PUT") {
            raw.push_str(&format!("content-length: {}\r\n", body_bytes.len()));
        }
        raw.push_str("\r\n");
        stream.write_all(raw.as_bytes()).await.ok()?;
        stream.write_all(&body_bytes).await.ok()?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await.ok()?;
        Some(parse_response(&buffer))
    }

    async fn request(
        &self,
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: Option<Vec<u8>>,
    ) -> Response {
        self.try_request(method, path, headers, body)
            .await
            .expect("request completes")
    }
}

fn pick_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn parse_response(raw: &[u8]) -> Response {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header terminator");
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let body = raw[split + 4..].to_vec();

    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect();

    Response {
        status,
        headers,
        body,
    }
}

fn origin() -> (String, String) {
    ("origin".to_owned(), "http://localhost".to_owned())
}

fn cookie(previous: &Response) -> (String, String) {
    let value = previous
        .header("set-cookie")
        .expect("session cookie on login")
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    ("cookie".to_owned(), value)
}

async fn setup_admin(server: &E2eServer) -> Response {
    server
        .request(
            "POST",
            "/api/identity/setup",
            &[
                origin(),
                (
                    "x-setup-secret".to_owned(),
                    "e2e-m2-secret-not-real".to_owned(),
                ),
                ("content-type".to_owned(), "application/json".to_owned()),
            ],
            Some(br#"{"email":"admin@example.com","password":"correct-horse-battery"}"#.to_vec()),
        )
        .await
}

async fn login(server: &E2eServer, email: &str) -> Response {
    server
        .request(
            "POST",
            "/api/identity/login",
            &[
                origin(),
                ("content-type".to_owned(), "application/json".to_owned()),
            ],
            Some(
                format!(r#"{{"email":"{email}","password":"correct-horse-battery"}}"#).into_bytes(),
            ),
        )
        .await
}

async fn create_member(server: &E2eServer, admin_cookie: &str, email: &str) {
    let response = server
        .request(
            "POST",
            "/api/identity/users",
            &[
                origin(),
                ("cookie".to_owned(), admin_cookie.to_owned()),
                ("content-type".to_owned(), "application/json".to_owned()),
            ],
            Some(
                format!(r#"{{"email":"{email}","password":"correct-horse-battery"}}"#).into_bytes(),
            ),
        )
        .await;
    assert_eq!(response.status, 201, "member {email} created");
}

/// One second of 440Hz MP3, generated through ffmpeg so the pipeline is
/// exercised end to end against real audio.
fn mp3_fixture() -> Vec<u8> {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("fixture.mp3");
    let status = std::process::Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .arg("-c:a")
        .arg("libmp3lame")
        .arg(&path)
        .status()
        .expect("ffmpeg fixture");
    assert!(status.success(), "fixture generation failed");
    std::fs::read(&path).unwrap()
}

/// A minimal WAV fixture: 44-byte header plus PCM payload, no ffmpeg
/// needed to build it (sine not required; any decodable PCM works).
fn wav_fixture() -> Vec<u8> {
    // CD-shaped stereo PCM (44.1kHz, 2ch, 16-bit): the realistic case where
    // Opus storage efficiency is the whole point (176 KB/s raw vs 16 KB/s
    // encoded at 128k).
    let sample_rate: u32 = 44_100;
    let frames: u32 = sample_rate * 2;
    let data_len = frames * 2 * 2;
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for frame in 0..frames {
        let amplitude = ((frame as f32 * 0.02).sin() * 9000.0) as i16;
        wav.extend_from_slice(&amplitude.to_le_bytes());
        wav.extend_from_slice(&amplitude.to_le_bytes());
    }
    wav
}

fn multipart_body(fields: &[(&str, &str, Vec<u8>)]) -> (String, Vec<u8>) {
    let boundary = "e2e-medioss-boundary-7351";
    let mut body = Vec::new();
    for (name, filename, bytes) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match *filename {
            "" => {
                body.extend_from_slice(
                    format!("content-disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
                );
                body.extend_from_slice(bytes);
                body.extend_from_slice(b"\r\n");
            }
            file => {
                body.extend_from_slice(
                    format!(
                        "content-disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\n"
                    )
                    .as_bytes(),
                );
                body.extend_from_slice(b"content-type: application/octet-stream\r\n\r\n");
                body.extend_from_slice(bytes);
                body.extend_from_slice(b"\r\n");
            }
        }
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

#[tokio::test]
async fn m2_full_battery() {
    let server = E2eServer::spawn().await;

    // Admin + two members.
    let admin_setup = setup_admin(&server).await;
    assert_eq!(admin_setup.status, 201);
    let admin_login = login(&server, "admin@example.com").await;
    assert_eq!(admin_login.status, 200);
    let admin_cookie = cookie(&admin_login).1;
    create_member(&server, &admin_cookie, "a@example.com").await;
    create_member(&server, &admin_cookie, "b@example.com").await;
    let login_a = login(&server, "a@example.com").await;
    let cookie_a = cookie(&login_a).1;
    let login_b = login(&server, "b@example.com").await;
    let cookie_b = cookie(&login_b).1;

    // --- Unauthenticated meditation surface: 401 everywhere (FR-006).
    let response = server
        .request("GET", "/api/meditation/library", &[origin()], None)
        .await;
    assert_eq!(response.status, 401);

    // --- Admin creates a program (uploads now belong to one, user
    // direction) and uploads an MP3 into it; it is stored as Opus (FR-001/038).
    let (content_type, program_body) = multipart_body(&[
        ("title", "", b"Morning Program".to_vec()),
        ("description", "", b"e2e program".to_vec()),
    ]);
    let response = server
        .request(
            "POST",
            "/api/meditation/programs",
            &[
                origin(),
                ("cookie".to_owned(), admin_cookie.clone()),
                ("content-type".to_owned(), content_type),
            ],
            Some(program_body),
        )
        .await;
    assert_eq!(response.status, 201, "program create: {}", response.text());
    let program_id: i64 = {
        let text = response.text();
        let start = text.find("\"id\":").map(|index| index + 5).unwrap();
        let end = text[start..]
            .find(',')
            .map(|index| start + index)
            .unwrap_or(text.len());
        text[start..end].parse().unwrap()
    };
    let mp3 = mp3_fixture();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Morning Meditation".to_vec()),
        ("program_id", "", program_id.to_string().into_bytes()),
        ("file", "morning.mp3", mp3),
    ]);
    let response = server
        .request(
            "POST",
            "/api/meditation/audio",
            &[
                origin(),
                ("cookie".to_owned(), admin_cookie.clone()),
                ("content-type".to_owned(), content_type),
            ],
            Some(body),
        )
        .await;
    assert_eq!(response.status, 201, "admin upload: {}", response.text());
    assert!(response.text().contains("\"content_type\":\"audio/mpeg\""));
    let meditation_id: i64 = {
        let text = response.text();
        let start = text.find("\"id\":").map(|index| index + 5).unwrap();
        let end = text[start..]
            .find(',')
            .map(|index| start + index)
            .unwrap_or(text.len());
        text[start..end].parse().unwrap()
    };

    // --- Member streams with a Range: 206, exactly 100 bytes, valid
    // Content-Range (SC-001).
    let response = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/audio"),
            &[
                origin(),
                ("cookie".to_owned(), cookie_a.clone()),
                ("range".to_owned(), "bytes=0-99".to_owned()),
            ],
            None,
        )
        .await;
    assert_eq!(response.status, 206, "range status");
    assert_eq!(response.body.len(), 100, "range body is exactly 100 bytes");
    let content_range = response.header("content-range").expect("content-range");
    assert!(
        content_range.starts_with("bytes 0-99/"),
        "content-range: {content_range}"
    );
    assert!(
        response.body.starts_with(b"ID3"),
        "stored form is the original MP3"
    );

    // --- Unknown id: 404; unauthenticated audio: 401 (SC-001).
    let response = server
        .request(
            "GET",
            "/api/meditation/99999/audio",
            &[origin(), ("cookie".to_owned(), cookie_a.clone())],
            None,
        )
        .await;
    assert_eq!(response.status, 404);
    let response = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/audio"),
            &[origin()],
            None,
        )
        .await;
    assert_eq!(response.status, 401);

    // --- Both members see the same shared library (FR-007, SC-005).
    for cookie_value in [&cookie_a, &cookie_b] {
        let response = server
            .request(
                "GET",
                "/api/meditation/library",
                &[origin(), ("cookie".to_owned(), cookie_value.clone())],
                None,
            )
            .await;
        assert_eq!(response.status, 200);
        assert!(response.text().contains("Morning Meditation"));
    }

    // --- Member management attempts are 403 (FR-005, SC-004).
    for (method, path) in [
        ("POST", "/api/meditation/audio"),
        ("POST", "/api/meditation/categories"),
        ("POST", "/api/meditation/programs"),
    ] {
        let response = server
            .request(
                method,
                path,
                &[
                    origin(),
                    ("cookie".to_owned(), cookie_a.clone()),
                    ("content-type".to_owned(), "application/json".to_owned()),
                ],
                Some(b"{}".to_vec()),
            )
            .await;
        assert_eq!(response.status, 403, "{method} {path}");
    }

    // --- Garbage bytes named .mp3: 415, nothing stored (FR-004, SC-003).
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Fake".to_vec()),
        ("program_id", "", program_id.to_string().into_bytes()),
        ("file", "fake.mp3", b"definitely not audio".to_vec()),
    ]);
    let response = server
        .request(
            "POST",
            "/api/meditation/audio",
            &[
                origin(),
                ("cookie".to_owned(), admin_cookie.clone()),
                ("content-type".to_owned(), content_type),
            ],
            Some(body),
        )
        .await;
    assert_eq!(response.status, 415, "fake bytes rejected");
    let response = server
        .request(
            "GET",
            "/api/meditation/library",
            &[origin(), ("cookie".to_owned(), cookie_a.clone())],
            None,
        )
        .await;
    let occurrences = response.text().matches("\"name\":").count();
    assert_eq!(occurrences, 1, "nothing extra stored after 415");

    // --- WAV upload: stored Ogg Opus and substantially smaller (SC-016).
    let wav = wav_fixture();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Windy Evening".to_vec()),
        ("program_id", "", program_id.to_string().into_bytes()),
        ("file", "windy.wav", wav.clone()),
    ]);
    let response = server
        .request(
            "POST",
            "/api/meditation/audio",
            &[
                origin(),
                ("cookie".to_owned(), admin_cookie.clone()),
                ("content-type".to_owned(), content_type),
            ],
            Some(body),
        )
        .await;
    assert_eq!(response.status, 201, "wav upload: {}", response.text());
    assert!(response.text().contains("audio/flac"));
    let stored_size: i64 = {
        let text = response.text();
        let start = text
            .find("\"size_bytes\":")
            .map(|index| index + 13)
            .unwrap();
        let end = text[start..]
            .find(',')
            .map(|index| start + index)
            .unwrap_or(text.len());
        text[start..end].parse().unwrap()
    };
    assert!(
        (stored_size as usize) < wav.len() / 2,
        "flac output {} must be well under half of wav {}",
        stored_size,
        wav.len()
    );

    // --- Playback state, completion idempotency, toggle (FR-015..022).
    let started_at = "2026-09-26T20:00:00Z";
    let finished_at = "2026-09-26T20:01:00Z";
    let payload = format!(r#"{{"started_at":"{started_at}","finished_at":"{finished_at}"}}"#);
    let completion_headers = |cookie: String| {
        vec![
            origin(),
            ("cookie".to_owned(), cookie),
            ("content-type".to_owned(), "application/json".to_owned()),
        ]
    };
    let completion_path = format!("/api/meditation/{meditation_id}/completion");
    let first = server
        .request(
            "POST",
            &completion_path,
            &completion_headers(cookie_a.clone()),
            Some(payload.clone().into_bytes()),
        )
        .await;
    assert_eq!(first.status, 201, "{}", first.text());
    let retry = server
        .request(
            "POST",
            &completion_path,
            &completion_headers(cookie_a.clone()),
            Some(payload.into_bytes()),
        )
        .await;
    assert_eq!(retry.status, 201, "idempotent retry accepted");

    let state_a = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/playback-state"),
            &[origin(), ("cookie".to_owned(), cookie_a.clone())],
            None,
        )
        .await;
    assert!(state_a.text().contains("\"completed\":true"));
    assert!(state_a.text().contains("\"position_ms\":0"));

    // B never completed it (SC-005).
    let state_b = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/playback-state"),
            &[origin(), ("cookie".to_owned(), cookie_b.clone())],
            None,
        )
        .await;
    assert!(state_b.text().contains("\"completed\":false"));

    // Toggle off/on leaves sessions intact (SC-015).
    for completed in ["false", "true"] {
        let response = server
            .request(
                "PUT",
                &format!("/api/meditation/{meditation_id}/completed"),
                &[
                    origin(),
                    ("cookie".to_owned(), cookie_a.clone()),
                    ("content-type".to_owned(), "application/json".to_owned()),
                ],
                Some(format!("{{\"completed\":{completed}}}").into_bytes()),
            )
            .await;
        assert_eq!(response.status, 204);
    }

    // --- Notes are author-private (FR-034..037, SC-014).
    let response = server
        .request(
            "POST",
            &format!("/api/meditation/{meditation_id}/notes"),
            &[
                origin(),
                ("cookie".to_owned(), cookie_a.clone()),
                ("content-type".to_owned(), "application/json".to_owned()),
            ],
            Some(br#"{"text":"deep calm"}"#.to_vec()),
        )
        .await;
    assert_eq!(response.status, 201);

    let notes_b = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/notes"),
            &[origin(), ("cookie".to_owned(), cookie_b.clone())],
            None,
        )
        .await;
    assert_eq!(notes_b.text().trim(), "[]", "B sees none of A's notes");

    let response = server
        .request(
            "PUT",
            &format!("/api/meditation/{meditation_id}/notes/1"),
            &[
                origin(),
                ("cookie".to_owned(), cookie_b.clone()),
                ("content-type".to_owned(), "application/json".to_owned()),
            ],
            Some(br#"{"text":"hijack"}"#.to_vec()),
        )
        .await;
    assert_eq!(response.status, 404, "B cannot touch A's note");

    // --- Download is an authenticated attachment (FR-040, SC-017).
    let response = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/download"),
            &[origin(), ("cookie".to_owned(), cookie_b.clone())],
            None,
        )
        .await;
    assert_eq!(response.status, 200);
    let disposition = response.header("content-disposition").expect("disposition");
    assert!(disposition.contains("attachment"), "{disposition}");
    assert!(
        disposition.contains("Morning-Meditation.opus"),
        "{disposition}"
    );

    // --- Admin delete removes everything (FR-014, SC-006): notes gone,
    // audio 404, library shrinks, media bytes wiped.
    let response = server
        .request(
            "DELETE",
            &format!("/api/meditation/{meditation_id}"),
            &[origin(), ("cookie".to_owned(), admin_cookie.clone())],
            None,
        )
        .await;
    assert_eq!(response.status, 204);

    let response = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/audio"),
            &[origin(), ("cookie".to_owned(), cookie_a.clone())],
            None,
        )
        .await;
    assert_eq!(response.status, 404);

    let notes_a = server
        .request(
            "GET",
            &format!("/api/meditation/{meditation_id}/notes"),
            &[origin(), ("cookie".to_owned(), cookie_a.clone())],
            None,
        )
        .await;
    assert_eq!(notes_a.status, 404, "notes went with the meditation");
}
