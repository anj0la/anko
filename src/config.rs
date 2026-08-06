use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Context, Result};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub github: GitHub,
    pub scan: Scan,
}

impl Config {
    fn new() -> Self {
        Config {
            github: GitHub::new(),
            scan: Scan::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GitHub {
    repo: String,
    token_env: String,
}

impl GitHub {
    fn new() -> Self {
        GitHub {
        repo: String::new(),
        token_env: String::from("TARO_GITHUB_TOKEN"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Scan {
    include: Vec<String>,
    exclude: Vec<String>,
    tags: Vec<String>,
}

impl Scan {
    fn new() -> Self {
        Scan {
            include: Vec::new(),
            exclude: vec![".git/**".to_string()],
            tags: vec!["TODO".to_string(), "BUG".to_string(), "HELP".to_string(), "DEPR".to_string()]
        }
    }
}

pub fn init() -> Result<()> {
    println!("Initializing Taro...");
    let config = Config::new();

    let toml_str = toml::to_string(&config)
        .context("failed to serialize default config to TOML")?;

    create_files(&toml_str)
        .context("failed to write taro.toml")?;

    Ok(())
}

fn create_files(contents: &str) -> Result<()> {
    fs::create_dir_all("./taro")
        .context("failed to create ./taro directory")?;

    if Path::new("taro.toml").exists() {
        anyhow::bail!("taro.toml already exists, refusing to overwrite. Delete it manually if you want to reinitialize.");
    }

    fs::write("taro.toml", contents)
        .context("failed to write taro.toml")?;

    fs::write("./taro/state.json", r#"{"issues": []}"#)
        .context("failed to write taro.toml")?;
    Ok(())
}