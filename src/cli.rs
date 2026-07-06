use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// bodydashboard - generate mobile-first HTML health reports from bodylog and nutlog data.
#[derive(Parser, Debug)]
#[command(name = "bodydashboard", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate health dashboard reports.
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReportAction {
    /// Generate an HTML health dashboard report.
    Create(CreateArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Number of days of data to include
    #[arg(short, long, default_value = "7")]
    pub days: u32,

    /// Directory where the report will be saved
    #[arg(short, long, default_value = ".")]
    pub output_dir: PathBuf,

    /// Report filename
    #[arg(long, default_value = "report.html")]
    pub name: String,
}