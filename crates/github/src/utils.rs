use std::path::PathBuf;

pub fn language_from_path(path_buf: &PathBuf) -> &str {
    match path_buf.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("ml") => "ocaml",
        Some("py") => "python",
        Some("js") => "javascript",
        Some("ts") => "typescript",
        Some("cpp") => "cpp",
        Some("c") => "c",
        Some("java") => "java",
        Some("cs") => "csharp",
        _ => "",
    }
}