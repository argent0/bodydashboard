mod cli;
mod data;
mod report;
mod stats;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

use cli::{Cli, Commands, CreateArgs, ReportAction};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Report { action } => match action {
            ReportAction::Create(args) => create_report(args),
        },
    }
}

fn create_report(args: CreateArgs) -> Result<(), String> {
    let output_path: PathBuf = args.output_dir.join(&args.name);

    let data = data::fetch_all(args.days)?;
    let html = report::generate_html(&data);

    fs::create_dir_all(&args.output_dir)
        .map_err(|e| format!("creating output directory: {e}"))?;
    fs::write(&output_path, &html).map_err(|e| format!("writing report: {e}"))?;

    println!(
        "Report written to {} ({} bytes)",
        output_path.display(),
        html.len()
    );

    Ok(())
}