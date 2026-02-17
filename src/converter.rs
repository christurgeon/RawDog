use std::path::Path;

use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::{ImageBuffer, Rgb, RgbImage};
use imagepipe::Pipeline;

/// Convert a single ARW file to JPEG.
pub fn convert_arw_to_jpeg(
    input: &Path,
    output: &Path,
    quality: u8,
    resize: Option<u32>,
) -> Result<()> {
    // Decode and process the raw file through imagepipe
    let mut pipeline = Pipeline::new_from_file(input)
        .map_err(|e| anyhow!("Failed to create pipeline for {}: {}", input.display(), e))?;

    let decoded = pipeline
        .output_8bit(None)
        .map_err(|e| anyhow!("Failed to process {}: {}", input.display(), e))?;

    let width = decoded.width as u32;
    let height = decoded.height as u32;

    // Build an RgbImage from the decoded data
    let img: RgbImage = ImageBuffer::from_raw(width, height, decoded.data)
        .context("Failed to construct image buffer from decoded data")?;

    // Optional resize (long edge)
    let final_img: ImageBuffer<Rgb<u8>, Vec<u8>> = if let Some(max_edge) = resize {
        let long_edge = width.max(height);
        if long_edge > max_edge {
            let scale = max_edge as f64 / long_edge as f64;
            let new_w = (width as f64 * scale).round() as u32;
            let new_h = (height as f64 * scale).round() as u32;
            image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    } else {
        img
    };

    // Encode to JPEG
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }

    let mut out_file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create output file {}", output.display()))?;

    let encoder = JpegEncoder::new_with_quality(&mut out_file, quality);
    final_img
        .write_with_encoder(encoder)
        .with_context(|| format!("Failed to encode JPEG to {}", output.display()))?;

    Ok(())
}
