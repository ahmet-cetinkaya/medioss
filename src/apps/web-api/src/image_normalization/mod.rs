mod vp8l_preflight;
mod webp_preflight;

use webp_preflight::preflight_webp;

use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, LazyLock,
    },
};

const MAX_DECODE_DIMENSION: u32 = 8_192;
const MAX_INPUT_PIXELS: u64 = 4_000_000;
// High enough that photos and flat artwork stay visually clean.
const WEBP_OUTPUT_QUALITY: f32 = 88.0;
const MAX_DECODE_ALLOC: u64 = 64 * 1024 * 1024;
// Brief contention queues; sustained flooding still gets `Busy`.
#[cfg(not(test))]
const NORMALIZATION_ADMISSION_WAIT: std::time::Duration = std::time::Duration::from_secs(2);
#[cfg(test)]
const NORMALIZATION_ADMISSION_WAIT: std::time::Duration = std::time::Duration::from_millis(200);
// Waiters hold the request body in memory, so only a few may queue.
const MAX_ADMISSION_WAITERS: usize = 4;
static ADMISSION_WAITERS: AtomicUsize = AtomicUsize::new(0);
// Decoding and encoding are CPU- and memory-heavy: one image at a time.
const CONCURRENT_NORMALIZATIONS: usize = 1;
static NORMALIZATION_WORKER: LazyLock<Arc<tokio::sync::Semaphore>> =
    LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(CONCURRENT_NORMALIZATIONS)));

#[cfg(test)]
pub(crate) static NORMALIZATION_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

#[derive(Debug, Clone, Copy)]
pub struct ImageSpec {
    pub maximum_dimension: u32,
    pub maximum_output_bytes: usize,
    pub square: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageNormalizationError {
    InvalidPayload,
    OutputTooLarge,
    WorkerFailed,
    Busy,
}

/// Decrements the waiter count on every exit, including cancellation.
struct WaiterSlot;

impl Drop for WaiterSlot {
    fn drop(&mut self) {
        ADMISSION_WAITERS.fetch_sub(1, Ordering::AcqRel);
    }
}

async fn admit() -> Result<tokio::sync::OwnedSemaphorePermit, ImageNormalizationError> {
    if ADMISSION_WAITERS.fetch_add(1, Ordering::AcqRel) >= MAX_ADMISSION_WAITERS {
        ADMISSION_WAITERS.fetch_sub(1, Ordering::AcqRel);
        return Err(ImageNormalizationError::Busy);
    }
    let _slot = WaiterSlot;
    tokio::time::timeout(
        NORMALIZATION_ADMISSION_WAIT,
        NORMALIZATION_WORKER.clone().acquire_owned(),
    )
    .await
    .map_err(|_| ImageNormalizationError::Busy)?
    .map_err(|_| ImageNormalizationError::WorkerFailed)
}

pub async fn normalize(
    bytes: Vec<u8>,
    specification: ImageSpec,
) -> Result<Vec<u8>, ImageNormalizationError> {
    let permit = admit().await?;

    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        normalize_blocking(&bytes, specification)
    })
    .await
    .map_err(|_| ImageNormalizationError::WorkerFailed)?
}

fn normalize_blocking(
    bytes: &[u8],
    specification: ImageSpec,
) -> Result<Vec<u8>, ImageNormalizationError> {
    validate_output_dimensions(specification)?;

    let format = image::guess_format(bytes).map_err(|_| ImageNormalizationError::InvalidPayload)?;
    if !matches!(
        format,
        image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP
    ) {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    if format == image::ImageFormat::WebP {
        preflight_webp(bytes)?;
    }

    let reader = image_reader(bytes)?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    validate_input_dimensions(width, height)?;

    let image = image_reader(bytes)?
        .decode()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    let image = if specification.square {
        let side = image.width().min(image.height());
        image
            .crop_imm(
                (image.width() - side) / 2,
                (image.height() - side) / 2,
                side,
                side,
            )
            .resize_to_fill(
                specification.maximum_dimension,
                specification.maximum_dimension,
                image::imageops::FilterType::Lanczos3,
            )
    } else {
        image.resize(
            specification.maximum_dimension,
            specification.maximum_dimension,
            image::imageops::FilterType::Lanczos3,
        )
    };

    let output = encode_lossy_webp(&image)?;
    if output.is_empty() || output.len() > specification.maximum_output_bytes {
        return Err(ImageNormalizationError::OutputTooLarge);
    }
    Ok(output)
}

/// The `image` crate's WebP encoder is lossless-only, which bloats photos, so
/// encoding goes through libwebp (vendored and built by the `webp` crate).
fn encode_lossy_webp(image: &image::DynamicImage) -> Result<Vec<u8>, ImageNormalizationError> {
    let (width, height) = (image.width(), image.height());
    let (rgba, rgb);
    let encoder = if image.color().has_alpha() {
        rgba = image.to_rgba8();
        webp::Encoder::from_rgba(&rgba, width, height)
    } else {
        rgb = image.to_rgb8();
        webp::Encoder::from_rgb(&rgb, width, height)
    };
    encoder
        .encode_simple(false, WEBP_OUTPUT_QUALITY)
        .map(|encoded| encoded.to_vec())
        .map_err(|_| ImageNormalizationError::WorkerFailed)
}

fn image_reader(
    bytes: &[u8],
) -> Result<image::ImageReader<Cursor<&[u8]>>, ImageNormalizationError> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_DIMENSION);
    limits.max_image_height = Some(MAX_DECODE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(limits);
    Ok(reader)
}

fn validate_input_dimensions(width: u32, height: u32) -> Result<(), ImageNormalizationError> {
    if width == 0
        || height == 0
        || width > MAX_DECODE_DIMENSION
        || height > MAX_DECODE_DIMENSION
        || pixel_count(width, height).is_none_or(|pixels| pixels > MAX_INPUT_PIXELS)
    {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(())
}

fn validate_output_dimensions(specification: ImageSpec) -> Result<(), ImageNormalizationError> {
    if specification.maximum_dimension == 0
        || pixel_count(
            specification.maximum_dimension,
            specification.maximum_dimension,
        )
        .is_none_or(|pixels| pixels > MAX_INPUT_PIXELS)
    {
        return Err(ImageNormalizationError::OutputTooLarge);
    }
    Ok(())
}

fn pixel_count(width: u32, height: u32) -> Option<u64> {
    u64::from(width).checked_mul(u64::from(height))
}

#[cfg(test)]
#[path = "mod.tests.rs"]
mod tests;
