mod data;
mod report;
mod stats;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bodydashboard", about = "Generate a health dashboard HTML report")]
struct Args {
    /// Number of days of data to include
    #[arg(short, long, default_value = "7")]
    days: u32,

    /// Directory where the report will be saved
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Report filename
    #[arg(long, default_value = "report.html")]
    name: String,
}

fn main() {
    let args = Args::parse();
    let output_path = args.output_dir.join(&args.name);

    let data = match data::fetch_all(args.days) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error fetching data: {e}");
            std::process::exit(1);
        }
    };

    let html = report::generate_html(&data);

    if let Err(e) = fs::create_dir_all(&args.output_dir) {
        eprintln!("Error creating output directory: {e}");
        std::process::exit(1);
    }

    if let Err(e) = fs::write(&output_path, &html) {
        eprintln!("Error writing report: {e}");
        std::process::exit(1);
    }

    println!(
        "Report written to {} ({} bytes)",
        output_path.display(),
        html.len()
    );
}