//! Upload limits are env-driven (MEDIOSS_MAX_UPLOAD_BYTES and friends).
//!
//! Drives the real `medioss` binary: a raised limit accepts a large valid MP3,
//! a lowered limit rejects with 413, and an invalid value aborts startup.

use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MIB: usize = 1024 * 1024;
const SETUP_SECRET: &str = "e2e-upload-limits-secret";
const PASSWORD: &str = "correct-horse-battery";
const BOUNDARY: &str = "e2e-upload-limits-boundary";
const HEALTH_DEADLINE: Duration = Duration::from_secs(15);

struct E2eServer {
    child: Child,
    port: u16,
    _data_dir: tempfile::TempDir,
}

impl Drop for E2eServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl E2eServer {
    async fn spawn(extra_env: &[(&str, &str)]) -> Self {
        let data_dir = tempfile::tempdir().unwrap();
        let port = pick_free_port();
        let child = server_command(data_dir.path(), port, extra_env)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn medioss");
        let server = Self {
            child,
            port,
            _data_dir: data_dir,
        };
        server.wait_healthy().await;
        server
    }

    async fn wait_healthy(&self) {
        let deadline = Instant::now() + HEALTH_DEADLINE;
        while Instant::now() < deadline {
            if self.request("GET", "/api/health", &[], Vec::new()).await.0 == 200 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        panic!("server did not become healthy");
    }

    /// Returns (status, body). The body is written from a separate task because
    /// the server may answer 413 and close before reading it all.
    async fn request(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> (u16, String) {
        let Ok(stream) = tokio::net::TcpStream::connect(("127.0.0.1", self.port)).await else {
            return (0, String::new());
        };
        let (mut reader, mut writer) = stream.into_split();
        let mut head = format!(
            "{method} {path} HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\ncontent-length: {}\r\n",
            body.len()
        );
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str("\r\n");
        let writing = tokio::spawn(async move {
            let _ = writer.write_all(head.as_bytes()).await;
            let _ = writer.write_all(&body).await;
            // Closing the write half early makes the server abort before answering.
            std::future::pending::<()>().await;
        });
        let mut raw = Vec::new();
        let _ = reader.read_to_end(&mut raw).await;
        writing.abort();
        parse_response(&raw)
    }

    async fn admin_cookie(&self) -> String {
        let (status, _) = self
            .request(
                "POST",
                "/api/identity/setup",
                &[
                    ("origin", "http://localhost"),
                    ("x-setup-secret", SETUP_SECRET),
                    ("content-type", "application/json"),
                ],
                format!(r#"{{"email":"admin@example.com","password":"{PASSWORD}"}}"#).into_bytes(),
            )
            .await;
        assert_eq!(status, 201, "admin setup");
        self.login_cookie().await
    }

    async fn login_cookie(&self) -> String {
        let stream_response = self.raw_login().await;
        stream_response
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("set-cookie")
                    .then(|| value.trim().split(';').next().unwrap().to_owned())
            })
            .expect("session cookie on login")
    }

    async fn raw_login(&self) -> String {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", self.port))
            .await
            .unwrap();
        let body = format!(r#"{{"email":"admin@example.com","password":"{PASSWORD}"}}"#);
        let raw = format!(
            "POST /api/identity/login HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\norigin: http://localhost\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(raw.as_bytes()).await.unwrap();
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await.unwrap();
        String::from_utf8_lossy(&buffer).into_owned()
    }

    async fn create_program(&self, cookie: &str) -> String {
        let (content_type, body) = multipart(&[("title", "", b"Limits".to_vec())]);
        let (status, text) = self
            .request(
                "POST",
                "/api/meditation/programs",
                &[
                    ("origin", "http://localhost"),
                    ("cookie", cookie),
                    ("content-type", &content_type),
                ],
                body,
            )
            .await;
        assert_eq!(status, 201, "program create: {text}");
        let start = text.find("\"id\":").unwrap() + 5;
        let end = text[start..].find(',').map_or(text.len(), |i| start + i);
        text[start..end].to_owned()
    }

    async fn upload_sound(&self, cookie: &str, audio: Vec<u8>) -> u16 {
        let (content_type, body) = multipart(&[
            ("name", "", b"Rain".to_vec()),
            ("category_id", "", b"1".to_vec()),
            ("file", "rain.mp3", audio),
        ]);
        self.request(
            "POST",
            "/api/soundscape/sounds",
            &[
                ("origin", "http://localhost"),
                ("cookie", cookie),
                ("content-type", &content_type),
            ],
            body,
        )
        .await
        .0
    }

    async fn upload_audio(&self, cookie: &str, program_id: &str, audio: Vec<u8>) -> u16 {
        let (content_type, body) = multipart(&[
            ("name", "", b"Large".to_vec()),
            ("program_id", "", program_id.as_bytes().to_vec()),
            ("file", "large.mp3", audio),
        ]);
        self.request(
            "POST",
            "/api/meditation/audio",
            &[
                ("origin", "http://localhost"),
                ("cookie", cookie),
                ("content-type", &content_type),
            ],
            body,
        )
        .await
        .0
    }
}

const EARLY_REJECTION_DEADLINE: Duration = Duration::from_secs(3);
const DECLARED_BODY_BYTES: usize = 200 * MIB;

impl E2eServer {
    /// Sends only the request head, declaring a body that never arrives, and
    /// keeps the socket open. Returns the status line's code, or 0 when the
    /// server stayed silent past the deadline (it was waiting for the body).
    async fn declared_status_without_body(&self, method: &str, path: &str) -> u16 {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", self.port))
            .await
            .unwrap();
        let head = format!(
            "{method} {path} HTTP/1.1\r\nhost: localhost\r\norigin: http://localhost\r\ncontent-type: multipart/form-data; boundary={BOUNDARY}\r\ncontent-length: {DECLARED_BODY_BYTES}\r\n\r\n"
        );
        stream.write_all(head.as_bytes()).await.unwrap();
        let mut buffer = vec![0u8; 1024];
        let read = tokio::time::timeout(EARLY_REJECTION_DEADLINE, stream.read(&mut buffer)).await;
        match read {
            Ok(Ok(count)) => parse_response(&buffer[..count]).0,
            _ => 0,
        }
    }
}

fn server_command(data_dir: &std::path::Path, port: u16, extra_env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_medioss"));
    command
        .env("MEDIOSS_DATA_DIR", data_dir)
        .env("SESSION_SECRET", SETUP_SECRET)
        .env("MEDIOSS_ORIGIN", "http://localhost")
        .env("SESSION_COOKIE_SECURE", "false")
        .env("PORT", port.to_string())
        .env("MEDIOSS_STATIC_DIR", "/nonexistent");
    for (name, value) in extra_env {
        command.env(name, value);
    }
    command
}

fn pick_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn parse_response(raw: &[u8]) -> (u16, String) {
    let text = String::from_utf8_lossy(raw);
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let body = text.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    (status, body.to_owned())
}

fn multipart(fields: &[(&str, &str, Vec<u8>)]) -> (String, Vec<u8>) {
    let mut body = Vec::new();
    for (name, filename, bytes) in fields {
        body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        let disposition = if filename.is_empty() {
            format!("content-disposition: form-data; name=\"{name}\"\r\n\r\n")
        } else {
            format!(
                "content-disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\ncontent-type: application/octet-stream\r\n\r\n"
            )
        };
        body.extend_from_slice(disposition.as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={BOUNDARY}"), body)
}

/// One second of 440Hz MP3 made by ffmpeg.
fn short_mp3() -> Vec<u8> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fixture.mp3");
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .args(["-c:a", "libmp3lame"])
        .arg(&path)
        .status()
        .expect("ffmpeg fixture");
    assert!(status.success(), "fixture generation failed");
    std::fs::read(&path).unwrap()
}

/// A valid MP3 of exactly `total_bytes`: a leading ID3v2 tag whose padding
/// takes the size, followed by a real one-second MP3. Decoders skip the
/// padding, so large files stay cheap to probe and convert.
fn mp3_of_size(total_bytes: usize) -> Vec<u8> {
    const ID3_HEADER_BYTES: usize = 10;
    let audio = short_mp3();
    let padding = total_bytes - audio.len() - ID3_HEADER_BYTES;
    let mut file = Vec::with_capacity(total_bytes);
    file.extend_from_slice(b"ID3\x03\x00\x00");
    file.extend_from_slice(&syncsafe(padding));
    file.resize(ID3_HEADER_BYTES + padding, 0);
    file.extend_from_slice(&audio);
    file
}

fn syncsafe(value: usize) -> [u8; 4] {
    [
        ((value >> 21) & 0x7f) as u8,
        ((value >> 14) & 0x7f) as u8,
        ((value >> 7) & 0x7f) as u8,
        (value & 0x7f) as u8,
    ]
}

#[tokio::test]
async fn lowered_limit_rejects_a_two_mib_upload_with_413() {
    let server = E2eServer::spawn(&[("MEDIOSS_MAX_UPLOAD_BYTES", &MIB.to_string())]).await;
    let cookie = server.admin_cookie().await;
    let program_id = server.create_program(&cookie).await;

    let status = server
        .upload_audio(&cookie, &program_id, mp3_of_size(2 * MIB))
        .await;

    assert_eq!(status, 413);
}

#[tokio::test]
async fn raised_limit_accepts_150_mb_and_rejects_201_mib() {
    let server = E2eServer::spawn(&[("MEDIOSS_MAX_UPLOAD_BYTES", &(200 * MIB).to_string())]).await;
    let cookie = server.admin_cookie().await;
    let program_id = server.create_program(&cookie).await;

    let accepted = server
        .upload_audio(&cookie, &program_id, mp3_of_size(150 * 1_000_000))
        .await;
    let rejected = server
        .upload_audio(&cookie, &program_id, mp3_of_size(201 * MIB))
        .await;

    assert_eq!(accepted, 201);
    assert_eq!(rejected, 413);
}

#[test]
fn invalid_limit_aborts_startup_naming_the_variable() {
    let data_dir = tempfile::tempdir().unwrap();
    let output = server_command(
        data_dir.path(),
        pick_free_port(),
        &[("MEDIOSS_MAX_COVER_BYTES", "0")],
    )
    .output()
    .expect("run medioss");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MEDIOSS_MAX_COVER_BYTES"));
}

#[tokio::test]
async fn declared_oversize_length_gets_413_before_any_body_is_sent() {
    let server = E2eServer::spawn(&[]).await;

    for (method, path) in [
        ("POST", "/api/meditation/audio"),
        ("POST", "/api/soundscape/sounds"),
        ("POST", "/api/meditation/programs"),
        ("PUT", "/api/meditation/programs/1/cover"),
    ] {
        let status = server.declared_status_without_body(method, path).await;
        assert_eq!(status, 413, "{method} {path}");
    }
}

#[tokio::test]
async fn sound_exactly_at_the_cap_is_not_rejected_for_multipart_overhead() {
    let server = E2eServer::spawn(&[("MEDIOSS_MAX_SOUND_UPLOAD_BYTES", &MIB.to_string())]).await;
    let cookie = server.admin_cookie().await;

    let at_cap = server.upload_sound(&cookie, mp3_of_size(MIB)).await;
    let over_cap = server.upload_sound(&cookie, mp3_of_size(MIB + 1)).await;

    // The category may not exist in this bare data dir, so the at-cap upload
    // can fail validation; what matters is that the envelope alone is not a 413.
    assert_ne!(at_cap, 413);
    assert_ne!(at_cap, 0);
    assert_eq!(over_cap, 413);
}
