use std::fs;
use std::path::Path;
use ignore::WalkBuilder;

use crate::parse::{self, TrackedTag};
use crate::hash;

#[derive(Debug)]
pub enum ScanError {
    Io(std::io::Error),
    InvalidUtf8,
}

impl From<std::io::Error> for ScanError {
    fn from(err: std::io::Error) -> Self {
        ScanError::Io(err)
    }
}

/// Walks root, respecting .gitignore + the hardcoded denylist,
/// reads each matching file, and returns all tags found.
/// Files that fail to read/parse are skipped and logged.
pub fn scan_tree(root: &Path) -> Result<Vec<TrackedTag>, ScanError> {
    let deny_list = [".git", "node_modules", "target", "vendor", "dist", "build"];

    let walker = WalkBuilder::new(root)
        .filter_entry(move |entry| {
            !deny_list.contains(&entry.file_name().to_string_lossy().as_ref())
        })
        .build();

    let mut tracked_tags: Vec<TrackedTag> = Vec::new();

    for result in walker {
        match result {
            Ok(entry) => {
                let is_file = entry.file_type().map_or(false, |ft| ft.is_file());
                if !is_file { // skips directories
                    continue;
                }
                match scan_file(entry.path()) {
                    Ok(mut tags) => tracked_tags.append(&mut tags),
                    Err(err) => {
                        eprintln!("warn: skipping {}: {:?}", entry.path().display(), err);
                    }
                }
            }
            Err(err) => eprintln!("error walking tree: {}", err),
        }
    }

    Ok(tracked_tags)

}

fn scan_file(path: &Path) -> Result<Vec<TrackedTag>, ScanError> {
    let contents = fs::read_to_string(path)?;

    let mut tags = Vec::new();

    for line in contents.lines() {
        if let Some((kind, labels, message)) = parse::parse_tag_line(line) {
            let hash = hash::compute_hash(&kind, &labels, &message, path);
            tags.push(TrackedTag {
                kind,
                labels,
                message,
                file: path.to_path_buf(),
                hash,
            });
        }
    }

    Ok(tags)
}