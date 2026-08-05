use clap::{Parser, Subcommand};
use crate::config::init;

#[derive(Parser)]
#[command(name = "taro")]
#[command(
    version,
    name = "taro",
    about = "A GitHub automation CLI that transforms code annotations into actionable issues.",
    long_about = "Taro scans your codebase for annotations like TODO and FIXME, then turns them into actionable GitHub issues. It helps developers track technical debt, manage unfinished work, and keep code annotations connected to their project workflow."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Taro
    Init,
    /// Preview tags and tracking status
    Scan,
    /// Sync tags with GitHub issues
    Sync,
    /// Close GitHub issues
    Close,
}

pub fn parse_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => init(),
        Commands::Scan => println!("Scanning..."),
        Commands::Sync => println!("Syncing tags..."),
        Commands::Close => println!("Closing issues..."),
    }
}