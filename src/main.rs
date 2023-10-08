use std::path::PathBuf;
use plotters::prelude::*;
use clap::Parser;
use statrs::distribution::{Beta, Continuous, ContinuousCDF};
use opener;
use tempfile;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t=0.7f64)]
    mean: f64,

    #[arg(long, default_value_t=0.2f64)]
    stddev: f64,

    #[arg(long, default_value_t=0f64)]
    min: f64,

    #[arg(long, default_value_t=1f64)]
    max: f64,

    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli { mean, stddev, min, max, out: output_path } = Cli::parse();

    let scale = max - min;
    let mean = (mean - min) / scale;
    let variance = (stddev / scale).powf(2f64);

    let (alpha, beta) = estimate_alpha_beta(mean, variance);

    let output_path = match output_path {
        Some(path) => path,
        None => tempfile::Builder::new().suffix(".png").tempfile()?.path().to_owned(),
    };

    let root = BitMapBackend::new(&output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let beta = Beta::new(alpha, beta)?;

    let y_max = (0..1000).map(|x| beta.pdf(x as f64 / 1000.0)).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Beta Distribution", ("Arial", 40).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min..max, 0f64..y_max)?;

    chart.configure_mesh().draw()?;

    let line = (0..1000).map(|x| {
        let x = x as f64 / 1000.0;
        let y = beta.pdf(x);
        (x * scale + min, y)
    });

    chart.draw_series(LineSeries::new(line, &RED))?;

    root.present()?;

    opener::open(&output_path)?;

    println!("90% confidence interval: {} to {}", beta.inverse_cdf(0.05) * scale + min, beta.inverse_cdf(0.95) * scale + min);

    Ok(())
}

fn estimate_alpha_beta(mean: f64, variance: f64) -> (f64, f64) {
    let common_factor = mean * (1f64 - mean) / variance - 1f64;
    let alpha = mean * common_factor;
    let beta = (1f64 - mean) * common_factor;
    (alpha, beta)
}

