// Mirrors meditation-infrastructure's TempFileStaging: the boundary rule forbids sharing it across modules.
//! Disk staging for uploads: each upload streams into a uniquely named file
//! under the staging root, so its size never counts against memory.
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use soundscape_application::ports::staged_upload::{
    StagedMedia, StagedSink, StagingError, UploadStaging,
};
use tempfile::TempPath;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const STAGED_FILE_PREFIX: &str = "upload-";

/// Resolves a staged address to its file under `root`. An address is a bare
/// file name; anything with a path component is refused.
pub fn resolve_staged_path(root: &Path, address: &str) -> Result<PathBuf, StagingError> {
    let mut components = Path::new(address).components();
    match (components.next(), components.next()) {
        (Some(std::path::Component::Normal(name)), None) => Ok(root.join(name)),
        _ => Err(StagingError::Io(format!(
            "invalid staged address {address:?}"
        ))),
    }
}

fn io_error(error: std::io::Error) -> StagingError {
    StagingError::Io(error.to_string())
}

/// [`UploadStaging`] over temp files in one directory.
#[derive(Debug, Clone)]
pub struct TempFileStaging {
    root: PathBuf,
}

impl TempFileStaging {
    pub fn new(staging_root: PathBuf) -> Self {
        Self { root: staging_root }
    }
}

struct TempFileSink {
    file: tokio::fs::File,
    path: TempPath,
}

#[async_trait]
impl StagedSink for TempFileSink {
    async fn append(&mut self, chunk: &[u8]) -> Result<(), StagingError> {
        self.file.write_all(chunk).await.map_err(io_error)
    }

    async fn keep(mut self: Box<Self>) -> Result<StagedMedia, StagingError> {
        self.file.flush().await.map_err(io_error)?;
        let size_bytes = self.file.metadata().await.map_err(io_error)?.len();
        let kept = self.path.keep().map_err(|error| io_error(error.error))?;
        let address = kept
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| StagingError::Io("staged file has no name".to_owned()))?;
        Ok(StagedMedia {
            address,
            size_bytes,
        })
    }
}

#[async_trait]
impl UploadStaging for TempFileStaging {
    #[cfg(unix)]
    async fn free_bytes(&self) -> Result<u64, StagingError> {
        use std::os::unix::ffi::OsStrExt;

        tokio::fs::create_dir_all(&self.root)
            .await
            .map_err(io_error)?;
        let path = std::ffi::CString::new(self.root.as_os_str().as_bytes())
            .map_err(|error| StagingError::Io(error.to_string()))?;
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: `path` is a valid NUL-terminated string and `stats` is a
        // writable statvfs; it is read only after the call reports success.
        let status = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };
        if status != 0 {
            return Err(io_error(std::io::Error::last_os_error()));
        }
        // SAFETY: statvfs returned 0, so it filled the struct.
        let stats = unsafe { stats.assume_init() };
        #[allow(clippy::unnecessary_cast)] // field widths differ across unix targets
        Ok(stats.f_bavail as u64 * stats.f_frsize as u64)
    }

    #[cfg(not(unix))]
    async fn free_bytes(&self) -> Result<u64, StagingError> {
        Err(StagingError::Unsupported)
    }

    async fn open(&self) -> Result<Box<dyn StagedSink>, StagingError> {
        tokio::fs::create_dir_all(&self.root)
            .await
            .map_err(io_error)?;
        let named = tempfile::Builder::new()
            .prefix(STAGED_FILE_PREFIX)
            .tempfile_in(&self.root)
            .map_err(io_error)?;
        let (file, path) = named.into_parts();
        Ok(Box::new(TempFileSink {
            file: tokio::fs::File::from_std(file),
            path,
        }))
    }

    async fn read_prefix(
        &self,
        media: &StagedMedia,
        max_bytes: usize,
    ) -> Result<Vec<u8>, StagingError> {
        let path = resolve_staged_path(&self.root, &media.address)?;
        let file = tokio::fs::File::open(path).await.map_err(io_error)?;
        let mut prefix = Vec::new();
        file.take(max_bytes as u64)
            .read_to_end(&mut prefix)
            .await
            .map_err(io_error)?;
        Ok(prefix)
    }

    async fn discard(&self, media: &StagedMedia) -> Result<(), StagingError> {
        let path = resolve_staged_path(&self.root, &media.address)?;
        match tokio::fs::remove_file(path).await {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(io_error(error)),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "staged_upload.tests.rs"]
mod tests;
