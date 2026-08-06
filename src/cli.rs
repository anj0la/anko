use clap::{Parser, Subcommand};
use crate::config;

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
    /// Initialize taro
    Init,
    /// Preview tags and tracking status -> so it gets all of the tags
    Scan,
    /// Sync tags with GitHub issues
    Sync,
    /// Close GitHub issues
    Close,
}

pub fn parse_cli() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init => config::init(),
        // Commands::Scan => println!("Scanning..."),
        //Commands::Sync => println!("Syncing tags..."),
        // Commands::Close => println!("Closing issues..."),
        _ => { eprintln!("unknown command"); std::process::exit(1); }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }

}