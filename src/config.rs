use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct Config {
    github: GitHub,
    scan: Scan,
}

impl Config {
    fn new() -> Self {
        Config {
            github: GitHub::new(),
            scan: Scan::new(),
        }
    }
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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

pub fn init() {
    println!("Initializing Taro...");
    let config = Config::new();

    match toml::to_string(&config) {
        Ok(toml_str) => to_file(&toml_str).unwrap_or_else(|e| eprintln!("Error writing file: {}", e)),
        Err(e) => eprintln!("Error serializing to TOML: {}", e),
    }
    println!("Finished initializing Taro.");
}

fn to_file(str: &str) -> std::io::Result<()> {
    fs::create_dir("./taro")?; // write empty state.JSON later
    fs::write("taro.toml", str)?;
    Ok(())
}