use std::path::PathBuf;
use std::sync::LazyLock;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind { Todo, Bug, Depr, }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedTag {
    pub kind: Kind,
    pub labels: Vec<String>,
    pub message: String,
    pub file: PathBuf,
    pub hash: String,
}

static LINE_REG: LazyLock<Regex> = LazyLock::new(|| { Regex::new(r"^\s*(?://|#)\s*(TODO|BUG|DEPR)\s*(?:\(([^)]*)\))?\s*:\s*(.+)$").unwrap() 
});
static BLOCK_REG: LazyLock<Regex> = LazyLock::new(|| { Regex::new(r"^\s*\(\*\s*(TODO|BUG|DEPR)\s*(?:\(([^)]*)\))?\s*:\s*(.+?)\s*\*\)\s*$").unwrap() 
});


/// Attempts to parse one line as a tagged comment
/// Returns None if the line doesn't match the pattern at all
pub fn parse_tag_line(line: &str) -> Option<(Kind, Vec<String>, String)> {
    let caps = LINE_REG.captures(line).or_else(|| BLOCK_REG.captures(line))?;

    let kind = match &caps[1] {
        "TODO" => Kind::Todo,
        "BUG" => Kind::Bug,
        "DEPR" => Kind::Depr,
        _ => unreachable!()
    };
    let labels = caps.get(2).
                map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(); // each label is its own string

    let message = caps[3].trim().to_string();
    Some((kind, labels, message))
}

// Tests

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn parses_slash_slash_with_labels() {
        let result = parse_tag_line("// TODO(lexer, parser): rewrite error recovery");
        assert_eq!(result, Some((Kind::Todo, vec!["lexer".into(), "parser".into()], "rewrite error recovery".into())));
    }

    #[test]
    fn parses_slash_slash_no_labels() {
        let result = parse_tag_line("// TODO: fix this");
        assert_eq!(result, Some((Kind::Todo, vec![], "fix this".into())));
    }

    #[test]
    fn parses_hash_style() {
        let result = parse_tag_line("# BUG(auth): token refresh race condition");
        assert_eq!(result, Some((Kind::Bug, vec!["auth".into()], "token refresh race condition".into())));
    }

    #[test]
    fn parses_ocaml_block_style() {
        let result = parse_tag_line("(* DEPR(api): remove v1 endpoint *)");
        assert_eq!(result, Some((Kind::Depr, vec!["api".into()], "remove v1 endpoint".into())));
    }

    #[test]
    fn rejects_non_matching_line() {
        let result = parse_tag_line("// just a regular comment");
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_missing_colon() {
        let result = parse_tag_line("// TODO fix this");
        assert_eq!(result, None);
    }
}

