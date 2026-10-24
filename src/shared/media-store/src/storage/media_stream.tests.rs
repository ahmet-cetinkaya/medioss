use tokio::io::{AsyncReadExt, BufReader};

use super::MediaStream;

#[tokio::test]
async fn reads_every_byte_of_the_wrapped_reader() {
    let mut stream = MediaStream::new(&b"hello blob"[..]);

    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();

    assert_eq!(bytes, b"hello blob");
}

#[tokio::test]
async fn an_empty_reader_yields_no_bytes() {
    let mut stream = MediaStream::new(&b""[..]);

    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();

    assert!(bytes.is_empty());
}

#[tokio::test]
async fn reader_hands_back_the_same_bytes_for_direct_driving() {
    let mut reader = MediaStream::new(BufReader::new(&b"abc"[..])).reader();

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await.unwrap();

    assert_eq!(bytes, b"abc");
}
