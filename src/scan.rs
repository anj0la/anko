use crate::config::Config;
use std::path::PathBuf;
use anyhow::{Context, Result};
use std::fs;

enum TagKind { Todo, Bug, Help, Depr }

struct Tag {
    kind: TagKind,
    labels: Vec<String>, 
    message: String,
    file: PathBuf,
    line: usize,
}

pub fn scan() -> Result<()> {
    println!("Scanning directories for tags...");

    let toml_str = fs::read_to_string("taro.toml")
        .context("failed to read taro.toml — did you run `taro init`?")?;

    let config: Config = toml::from_str(&toml_str)
        .context("taro.toml exists but isn't valid TOML")?;

    scan_dirs(&config)?;
    Ok(())
}

fn scan_dirs(config: &Config) -> Result<()> {
    // scanning logic
    // start from the includes
    // config.scan.includes -> shows us which files / folders to scan
    // config.scan.excludes -> for each include file, we need to check that it's not excluded
    Ok(())
}