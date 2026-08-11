use sha2::{Digest, Sha256};
use std::path::Path;

use crate::parse::Kind;

/// Computes a hash using the SHA256 algorithm.
pub fn compute_hash(kind: &Kind, labels: &[String], message: &str, file: &Path) -> String {
    let mut hasher = Sha256::new();
    let mut sorted_list = labels.to_vec();
    sorted_list.sort();

    hasher.update(format!("{:?}", kind));
    hasher.update(b"\0");
    hasher.update(sorted_list.join("\0"));
    hasher.update(b"\0");
    hasher.update(message);
    hasher.update(b"\0");
    hasher.update(file.to_string_lossy().as_bytes());
    hasher.update(b"\0");
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod hash_tests {
    use super::*;

    fn sample_labels() -> Vec<String> {
        vec!["lexer".to_string(), "parser".to_string()]
    }

    #[test]
    fn same_input_produces_same_hash() {
        let file = Path::new("src/parser.rs");
        let h1 = compute_hash(
            &Kind::Todo,
            &sample_labels(),
            "rewrite error recovery",
            file,
        );
        let h2 = compute_hash(
            &Kind::Todo,
            &sample_labels(),
            "rewrite error recovery",
            file,
        );
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_kind_produces_different_hash() {
        let file = Path::new("src/parser.rs");
        let h_todo = compute_hash(&Kind::Todo, &sample_labels(), "same message", file);
        let h_bug = compute_hash(&Kind::Bug, &sample_labels(), "same message", file);
        assert_ne!(h_todo, h_bug);
    }

    #[test]
    fn different_message_produces_different_hash() {
        let file = Path::new("src/parser.rs");
        let h1 = compute_hash(&Kind::Todo, &sample_labels(), "message one", file);
        let h2 = compute_hash(&Kind::Todo, &sample_labels(), "message two", file);
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_file_produces_different_hash() {
        let h1 = compute_hash(
            &Kind::Todo,
            &sample_labels(),
            "same message",
            Path::new("src/a.rs"),
        );
        let h2 = compute_hash(
            &Kind::Todo,
            &sample_labels(),
            "same message",
            Path::new("src/b.rs"),
        );
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_labels_produces_different_hash() {
        let file = Path::new("src/parser.rs");
        let h1 = compute_hash(
            &Kind::Todo,
            &vec!["lexer".to_string()],
            "same message",
            file,
        );
        let h2 = compute_hash(
            &Kind::Todo,
            &vec!["parser".to_string()],
            "same message",
            file,
        );
        assert_ne!(h1, h2);
    }

    #[test]
    fn empty_labels_does_not_panic_and_is_stable() {
        let file = Path::new("src/parser.rs");
        let empty: Vec<String> = vec![];
        let h1 = compute_hash(&Kind::Todo, &empty, "no labels here", file);
        let h2 = compute_hash(&Kind::Todo, &empty, "no labels here", file);
        assert_eq!(h1, h2);
    }

    #[test]
    fn label_order_does_not_affect_hash() {
        let file = Path::new("src/parser.rs");
        let h1 = compute_hash(
            &Kind::Todo,
            &vec!["a".to_string(), "b".to_string()],
            "msg",
            file,
        );
        let h2 = compute_hash(
            &Kind::Todo,
            &vec!["b".to_string(), "a".to_string()],
            "msg",
            file,
        );
        assert_eq!(
            h1, h2,
            "label order should not affect the hash now that labels are sorted"
        );
    }

    #[test]
    fn output_is_64_char_lowercase_hex() {
        let file = Path::new("src/parser.rs");
        let h = compute_hash(&Kind::Todo, &sample_labels(), "check format", file);
        assert_eq!(h.len(), 64); // SHA-256 -> 32 bytes -> 64 hex chars
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn single_comma_label_differs_from_two_labels() {
        let file = Path::new("src/parser.rs");
        let one_weird_label = vec!["lexer,parser".to_string()];
        let two_normal_labels = vec!["lexer".to_string(), "parser".to_string()];
        let h1 = compute_hash(&Kind::Todo, &one_weird_label, "fix", file);
        let h2 = compute_hash(&Kind::Todo, &two_normal_labels, "fix", file);
        assert_ne!(
            h1, h2,
            "a single label containing a comma should not collide with two separate labels"
        );
    }

    #[test]
    fn empty_labels_differs_from_message_starting_where_labels_would_end() {
        let file = Path::new("src/parser.rs");
        let empty: Vec<String> = vec![];
        let h1 = compute_hash(&Kind::Todo, &empty, "x", file);
        let h2 = compute_hash(&Kind::Todo, &vec!["x".to_string()], "", file);
        assert_ne!(
            h1, h2,
            "field boundaries must not blur even when adjacent fields are empty/short"
        );
    }
}
