mod converter;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{bail, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

#[derive(Parser)]
#[command(name = "rawdog", about = "Raw files in, JPEGs out. No Lightroom, no fuss.")]
struct Cli {
    /// One or more ARW files or directories containing ARW files
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output directory (defaults to same directory as each input file)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// JPEG quality (1-100)
    #[arg(short, long, default_value_t = 92, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,

    /// Resize long edge to this many pixels, preserving aspect ratio
    #[arg(short, long)]
    resize: Option<u32>,

    /// Overwrite existing output files (default: skip if exists)
    #[arg(long)]
    overwrite: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Collect all ARW files from inputs
    let files = collect_arw_files(&cli.input)?;

    if files.is_empty() {
        bail!("No ARW files found in the provided inputs.");
    }

    println!("Found {} ARW file(s)", files.len());

    let succeeded = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("invalid progress bar template")
            .progress_chars("=> "),
    );

    files.par_iter().for_each(|input_path| {
        let output_path = make_output_path(input_path, cli.output.as_deref());

        // Skip if exists and overwrite is not set
        if !cli.overwrite && output_path.exists() {
            skipped.fetch_add(1, Ordering::Relaxed);
            pb.inc(1);
            return;
        }

        match converter::convert_arw_to_jpeg(input_path, &output_path, cli.quality, cli.resize) {
            Ok(()) => {
                succeeded.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                pb.suspend(|| {
                    eprintln!("Error converting {}: {:#}", input_path.display(), e);
                });
                failed.fetch_add(1, Ordering::Relaxed);
            }
        }

        pb.inc(1);
    });

    pb.finish_and_clear();

    let s = succeeded.load(Ordering::Relaxed);
    let f = failed.load(Ordering::Relaxed);
    let sk = skipped.load(Ordering::Relaxed);

    println!("{s} succeeded, {f} failed, {sk} skipped");

    Ok(())
}

/// Collect all `.arw`/`.ARW` files from the given paths.
/// Paths can be individual files or directories (non-recursive).
fn collect_arw_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in inputs {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let p = entry.path();
                if is_arw(&p) {
                    files.push(p);
                }
            }
        } else if path.is_file() {
            files.push(path.clone());
        } else {
            eprintln!("Warning: {} does not exist, skipping", path.display());
        }
    }

    files.sort();
    Ok(files)
}

fn is_arw(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("arw"))
        .unwrap_or(false)
}

/// Build the output JPEG path for a given input ARW file.
fn make_output_path(input: &Path, output_dir: Option<&Path>) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default();
    let jpeg_name = format!("{}.jpg", stem.to_string_lossy());

    match output_dir {
        Some(dir) => dir.join(jpeg_name),
        None => input.with_file_name(jpeg_name),
    }
}
