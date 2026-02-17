use std::path::Path;

use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::tiff::TiffEncoder;
use image::{ImageBuffer, Rgb, RgbImage};
use imagepipe::Pipeline;

use crate::OutputFormat;

/// Convert a single ARW file to the specified output format.
pub fn convert_arw(
    input: &Path,
    output: &Path,
    format: OutputFormat,
    quality: u8,
    resize: Option<u32>,
) -> Result<()> {
    let mut pipeline = Pipeline::new_from_file(input)
        .map_err(|e| anyhow!("Failed to create pipeline for {}: {}", input.display(), e))?;

    // Ensure output directory exists
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }

    match format {
        OutputFormat::Jpeg => encode_jpeg(&mut pipeline, input, output, quality, resize),
        OutputFormat::Tiff => encode_16bit(&mut pipeline, input, output, resize, Format16::Tiff),
        OutputFormat::Png => encode_16bit(&mut pipeline, input, output, resize, Format16::Png),
    }
}

fn encode_jpeg(
    pipeline: &mut Pipeline,
    input: &Path,
    output: &Path,
    quality: u8,
    resize: Option<u32>,
) -> Result<()> {
    let decoded = pipeline
        .output_8bit(None)
        .map_err(|e| anyhow!("Failed to process {}: {}", input.display(), e))?;

    let width = decoded.width as u32;
    let height = decoded.height as u32;

    let img: RgbImage = ImageBuffer::from_raw(width, height, decoded.data)
        .context("Failed to construct image buffer from decoded data")?;

    let final_img = maybe_resize_8bit(img, resize);

    let mut out_file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create output file {}", output.display()))?;

    let encoder = JpegEncoder::new_with_quality(&mut out_file, quality);
    final_img
        .write_with_encoder(encoder)
        .with_context(|| format!("Failed to encode JPEG to {}", output.display()))?;

    Ok(())
}

enum Format16 {
    Tiff,
    Png,
}

fn encode_16bit(
    pipeline: &mut Pipeline,
    input: &Path,
    output: &Path,
    resize: Option<u32>,
    fmt: Format16,
) -> Result<()> {
    let decoded = pipeline
        .output_16bit(None)
        .map_err(|e| anyhow!("Failed to process {}: {}", input.display(), e))?;

    let width = decoded.width as u32;
    let height = decoded.height as u32;

    let img: ImageBuffer<Rgb<u16>, Vec<u16>> =
        ImageBuffer::from_raw(width, height, decoded.data)
            .context("Failed to construct 16-bit image buffer from decoded data")?;

    let final_img = maybe_resize_16bit(img, resize);

    let mut out_file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create output file {}", output.display()))?;

    let label = match fmt {
        Format16::Tiff => "TIFF",
        Format16::Png => "PNG",
    };

    match fmt {
        Format16::Tiff => {
            let encoder = TiffEncoder::new(&mut out_file);
            final_img
                .write_with_encoder(encoder)
                .with_context(|| format!("Failed to encode {} to {}", label, output.display()))?;
        }
        Format16::Png => {
            let encoder = PngEncoder::new(&mut out_file);
            final_img
                .write_with_encoder(encoder)
                .with_context(|| format!("Failed to encode {} to {}", label, output.display()))?;
        }
    }

    Ok(())
}

fn maybe_resize_8bit(
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    resize: Option<u32>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    if let Some(max_edge) = resize {
        let (width, height) = img.dimensions();
        let long_edge = width.max(height);
        if long_edge > max_edge {
            let scale = max_edge as f64 / long_edge as f64;
            let new_w = (width as f64 * scale).round() as u32;
            let new_h = (height as f64 * scale).round() as u32;
            return image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Lanczos3);
        }
    }
    img
}

fn maybe_resize_16bit(
    img: ImageBuffer<Rgb<u16>, Vec<u16>>,
    resize: Option<u32>,
) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
    if let Some(max_edge) = resize {
        let (width, height) = img.dimensions();
        let long_edge = width.max(height);
        if long_edge > max_edge {
            let scale = max_edge as f64 / long_edge as f64;
            let new_w = (width as f64 * scale).round() as u32;
            let new_h = (height as f64 * scale).round() as u32;
            return image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Lanczos3);
        }
    }
    img
}
