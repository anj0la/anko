use std::fs;
use std::path::Path;
use ignore::WalkBuilder;

use crate::parse::{self, Kind, TrackedTag};
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
pub fn scan_tree(root: &Path) -> Vec<TrackedTag> {
    let deny_list = [".git", "node_modules", "target", "vendor", "dist", "build"];

    let walker = WalkBuilder::new(root)
        .require_git(false)
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
                match scan_file(entry.path(), root) {
                    Ok(mut tags) => tracked_tags.append(&mut tags),
                    Err(err) => {
                        eprintln!("warn: skipping {}: {:?}", entry.path().display(), err);
                    }
                }
            }
            Err(err) => eprintln!("error walking tree: {}", err),
        }
    }

    tracked_tags

}

fn scan_file(path: &Path, root: &Path) -> Result<Vec<TrackedTag>, ScanError> {
    let contents = fs::read_to_string(path)?;
    let rel_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();

    let mut tags = Vec::new();
    for (i, line) in contents.lines().enumerate() {
        if let Some((kind, labels, message)) = parse::parse_tag_line(line) {
            let hash = hash::compute_hash(&kind, &labels, &message, &rel_path);
            tags.push(TrackedTag {
                kind,
                labels,
                message,
                file: rel_path.clone(),
                line: i + 1, // 1-indexed
                hash,
            });
        }
    }
    Ok(tags)
}

#[cfg(test)]
mod scan_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn single_file_single_tag() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "// TODO(parser): fix this\nfn main() {}\n",
        )
        .unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].kind, Kind::Todo);
        assert_eq!(tags[0].message, "fix this");
    }

    #[test]
    fn single_file_multiple_tags() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "// TODO: first\nfn main() {}\n// BUG: second\n",
        )
        .unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].message, "first");
        assert_eq!(tags[1].message, "second");
    }

    #[test]
    fn multiple_files_aggregate_across_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "// TODO: in file a\n").unwrap();
        fs::write(dir.path().join("b.rs"), "// TODO: in file b\n").unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 2);
        let messages: Vec<&str> = tags.iter().map(|t| t.message.as_str()).collect();
        assert!(messages.contains(&"in file a"));
        assert!(messages.contains(&"in file b"));
    }

    #[test]
    fn file_with_no_tags_contributes_nothing() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n// just a comment\n").unwrap();

        let tags = scan_tree(dir.path());

        assert!(tags.is_empty());
    }

    #[test]
    fn empty_directory_returns_empty_vec() {
        let dir = tempdir().unwrap();

        let tags = scan_tree(dir.path());

        assert!(tags.is_empty());
    }

    #[test]
    fn denylisted_directory_is_excluded() {
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        fs::write(target_dir.join("generated.rs"), "// TODO: should be ignored\n").unwrap();
        fs::write(dir.path().join("main.rs"), "// TODO: should be found\n").unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].message, "should be found");
    }

    #[test]
    fn gitignored_file_is_excluded() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "ignored.rs\n").unwrap();
        fs::write(dir.path().join("ignored.rs"), "// TODO: should be ignored\n").unwrap();
        fs::write(dir.path().join("main.rs"), "// TODO: should be found\n").unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].message, "should be found");
    }

    #[test]
    fn nested_directories_are_scanned() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("src").join("parser");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("lexer.rs"), "// TODO: nested tag\n").unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].message, "nested tag");
    }

    #[test]
    fn each_tag_has_correct_hash() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "// TODO(x): check hash\n").unwrap();

        let tags = scan_tree(dir.path());

        assert_eq!(tags.len(), 1);
        let expected_hash = hash::compute_hash(
            &Kind::Todo,
            &vec!["x".to_string()],
            "check hash",
            &tags[0].file,
        );
        assert_eq!(tags[0].hash, expected_hash);
    }

    // --- scan_file, tested directly since it's private ---

    #[test]
    fn scan_file_returns_empty_vec_for_no_tags() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("main.rs");
        fs::write(&path, "fn main() {}\n").unwrap();

        let tags = scan_file(&path, dir.path()).unwrap();

        assert!(tags.is_empty());
    }

    #[test]
    fn scan_file_returns_error_for_nonexistent_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist.rs");

        let result = scan_file(&path, dir.path());

        assert!(result.is_err());
    }
}