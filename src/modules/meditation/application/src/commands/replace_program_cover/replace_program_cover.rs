use kernel::UserId;

/// Replaces a program's cover with already-validated, normalized WebP bytes.
#[derive(Debug)]
pub struct ReplaceProgramCover {
    #[allow(dead_code)]
    pub requester_id: UserId,
    pub program_id: i64,
    pub bytes: Vec<u8>,
}
