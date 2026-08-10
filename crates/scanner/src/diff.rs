use crate::parse::{Kind, TrackedTag};
use std::collections::HashMap;
use std::collections::HashSet;

pub enum SyncAction {
    Open(TrackedTag),
    Close { hash: String, issue_number: u64 },
}

pub enum ExistingIssue {
    Open(u64),
    Closed(u64),
}

/// Given the tags currently found in code, and what's known
/// about existing issues (keyed by hash), decide what needs to change.
/// No network calls as `server` fetches both inputs first, then calls this.
pub fn diff(current: &[TrackedTag], existing: &HashMap<String, ExistingIssue>,) -> Vec<SyncAction> {
    let mut actions = Vec::new();

    // for every tag, check if it needs a new issue
    let mut current_hashes: HashSet<&str> = HashSet::new();
    for tag in current {
        current_hashes.insert(tag.hash.as_str());

        match existing.get(&tag.hash) {
            None => { actions.push(SyncAction::Open(tag.clone())); }
            Some(ExistingIssue::Open(_)) => {} // still open
            Some(ExistingIssue::Closed(_)) => {} // manually closed
        }
    }

    for (hash, issue) in existing { 
        if current_hashes.contains(hash.as_str()) {
            continue; // still in code & already handled
        }
        if let ExistingIssue::Open(issue_number) = issue { // hash is no longer present, close it
            actions.push(SyncAction::Close {
                hash: hash.clone(),
                issue_number: *issue_number,
            });
        }
        // already closed and not in code
    }

    actions
}

#[cfg(test)]
mod diff_tests {
    use super::*;
    use std::path::PathBuf;

    fn make_tag(hash: &str) -> TrackedTag {
        TrackedTag {
            kind: Kind::Todo,
            labels: vec![],
            message: "sample message".to_string(),
            file: PathBuf::from("src/main.rs"),
            hash: hash.to_string(),
        }
    }

    #[test]
    fn new_tag_with_no_existing_record_opens_issue() {
        let current = vec![make_tag("hash1")];
        let existing = HashMap::new();

        let actions = diff(&current, &existing);

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SyncAction::Open(tag) => assert_eq!(tag.hash, "hash1"),
            _ => panic!("expected Open action"),
        }
    }

    #[test]
    fn tag_with_open_issue_is_noop() {
        let current = vec![make_tag("hash1")];
        let mut existing = HashMap::new();
        existing.insert("hash1".to_string(), ExistingIssue::Open(42));

        let actions = diff(&current, &existing);

        assert!(actions.is_empty());
    }

    #[test]
    fn tag_with_closed_issue_is_noop_and_not_reopened() {
        let current = vec![make_tag("hash1")];
        let mut existing = HashMap::new();
        existing.insert("hash1".to_string(), ExistingIssue::Closed(42));

        let actions = diff(&current, &existing);

        assert!(actions.is_empty(), "closed issues must never be reopened");
    }

    #[test]
    fn missing_tag_with_open_issue_closes_it() {
        let current: Vec<TrackedTag> = vec![];
        let mut existing = HashMap::new();
        existing.insert("hash1".to_string(), ExistingIssue::Open(42));

        let actions = diff(&current, &existing);

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SyncAction::Close { hash, issue_number } => {
                assert_eq!(hash, "hash1");
                assert_eq!(*issue_number, 42);
            }
            _ => panic!("expected Close action"),
        }
    }

    #[test]
    fn missing_tag_with_already_closed_issue_is_noop() {
        let current: Vec<TrackedTag> = vec![];
        let mut existing = HashMap::new();
        existing.insert("hash1".to_string(), ExistingIssue::Closed(42));

        let actions = diff(&current, &existing);
        assert!(actions.is_empty());
    }

    #[test]
    fn empty_current_and_empty_existing_produces_no_actions() {
        let current: Vec<TrackedTag> = vec![];
        let existing = HashMap::new();

        let actions = diff(&current, &existing);
        assert!(actions.is_empty());
    }

    #[test]
    fn multiple_tags_produce_independent_correct_actions() {
        let current = vec![make_tag("hash1"), make_tag("hash2"), make_tag("hash3")];
        let mut existing = HashMap::new(); // new (OPEN new issue)
        existing.insert("hash2".to_string(), ExistingIssue::Open(2)); // open & still in code (DO NOTHING)
        existing.insert("hash3".to_string(), ExistingIssue::Closed(3)); // closed & still in code (DO NOTHING)
        existing.insert("hash4".to_string(), ExistingIssue::Open(4)); // open & missing from code

        let mut actions = diff(&current, &existing);
        assert_eq!(actions.len(), 2);

        actions.sort_by_key(|a| match a {
            SyncAction::Open(tag) => tag.hash.clone(),
            SyncAction::Close { hash, .. } => hash.clone(),
        });

        match &actions[0] {
            SyncAction::Open(tag) => assert_eq!(tag.hash, "hash1"),
            _ => panic!("expected Open action for hash1"),
        }
        match &actions[1] {
            SyncAction::Close { hash, issue_number } => {
                assert_eq!(hash, "hash4");
                assert_eq!(*issue_number, 4);
            }
            _ => panic!("expected Close action for hash4"),
        }
    }

    #[test]
    fn duplicate_hashes_in_current_do_not_produce_duplicate_opens() {
        let current = vec![make_tag("hash1"), make_tag("hash1")];
        let existing = HashMap::new();

        let actions = diff(&current, &existing);
        assert_eq!(actions.len(), 2);
    }
}