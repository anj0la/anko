use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use std::sync::Arc;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use std::collections::HashMap;
use std::fs;

use github::GitHubApp;
use scanner::diff::{ExistingIssue, SyncAction, diff};
use scanner::parse::TrackedTag;
use scanner::scan::scan_tree;
use store::Database;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct AppState {
    pub webhook_secret: String,
    pub db: Arc<Database>,
    pub github: Arc<GitHubApp>,
}

#[derive(serde::Deserialize)]
struct PushEvent {
    installation: Installation,
    repository: Repository,
    r#ref: String,
    after: String,
    deleted: bool,
}

#[derive(serde::Deserialize)]
struct Installation {
    id: u64,
}

#[derive(serde::Deserialize)]
struct Repository {
    name: String,
    owner: Owner,
    default_branch: String,
}

#[derive(serde::Deserialize)]
struct Owner {
    login: String,
}

fn verify_signature(secret: &str, body: &[u8], signature_header: &str) -> bool {
    let Some(sig_hex) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    let Ok(sig_bytes) = hex::decode(sig_hex) else {
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };

    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}

#[axum::debug_handler]
pub async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_signature(&state.webhook_secret, &body, signature) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if event_type != "push" {
        return Ok(StatusCode::OK);
    }

    let event: PushEvent = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    const ZERO_SHA: &str = "0000000000000000000000000000000000000000";
    let expected_ref = format!("refs/heads/{}", event.repository.default_branch);
    if event.r#ref != expected_ref || event.deleted || event.after == ZERO_SHA {
        return Ok(StatusCode::OK);
    }

    let installation_id = octocrab::models::InstallationId(event.installation.id);
    let owner = event.repository.owner.login;
    let repo = event.repository.name;
    let commit_sha = event.after;

    sync_repo(&state, installation_id, &owner, &repo, &commit_sha)
        .await
        .map_err(|e| {
            eprintln!("sync failed for {owner}/{repo}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

async fn checkout_and_scan(
    client: &octocrab::Octocrab,
    owner: &str,
    repo: &str,
    commit_sha: &str,
) -> Result<Vec<TrackedTag>, BoxError> {
    let tmp_dir = tempfile::tempdir()?;

    github::download_repo_tarball(client, owner, repo, commit_sha.to_string(), tmp_dir.path())
        .await?;

    let repo_root = fs::read_dir(tmp_dir.path())?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir())
        .map(|e| e.path())
        .ok_or("tarball had no top-level directory")?;

    let tags = scan_tree(&repo_root);
    Ok(tags)
}

pub async fn sync_repo(
    state: &AppState,
    installation_id: octocrab::models::InstallationId,
    owner: &str,
    repo: &str,
    commit_sha: &str,
) -> Result<(), BoxError> {
    let client = state.github.installation_client(installation_id).await?;
    let current = checkout_and_scan(&client, owner, repo, commit_sha).await?;

    let mut existing: HashMap<String, ExistingIssue> = HashMap::new();
    for tag in &current {
        if let Some(issue_number) = state.db.get(owner, repo, &tag.hash).await? {
            match state
                .github
                .get_issue(installation_id, owner, repo, issue_number)
                .await?
            {
                Some(issue_state) => {
                    existing.insert(tag.hash.clone(), issue_state);
                }
                None => {}
            }
        }
    }

    let actions = diff(&current, &existing);
    for action in actions {
        match action {
            SyncAction::Open(tag) => {
                let issue_number = state
                    .github
                    .create_issue(installation_id, owner, repo, &tag)
                    .await?;
                state.db.set(owner, repo, &tag.hash, issue_number).await?;
            }
            SyncAction::Close {
                hash: _,
                issue_number,
            } => {
                state
                    .github
                    .close_issue(installation_id, owner, repo, issue_number)
                    .await?;
            }
        }
    }

    Ok(())
}
