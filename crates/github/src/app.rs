use octocrab::{Octocrab, models::AppId, models::InstallationId, models::IssueState};
use jsonwebtoken::EncodingKey;
use secrecy::ExposeSecret;
use std::fs;

use scanner::{diff::ExistingIssue, parse::{TrackedTag, Kind}};

const BOT_LABEL: &str = "anko";

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct GitHubApp {
    pub client: Octocrab,
}

impl GitHubApp {
    pub fn new(app_id: u64, private_key_path: &str) -> Result<Self, BoxError> {
        let key_contents = fs::read_to_string(private_key_path)?;
        let encoding_key = EncodingKey::from_rsa_pem(key_contents.as_bytes())?;

        // installing app
        let app_client = Octocrab::builder()
            .app(AppId(app_id), encoding_key)
            .build()?;

        Ok(Self {
            client: app_client,
        })
    }

    /// Exchanges the App's JWT for a short-lived installation access
    /// token, returned as a raw string (needed for the git clone URL,
    /// not just an authenticated Octocrab client).
    pub async fn installation_token(&self, id: InstallationId) -> Result<String, BoxError> {
        let (_client, token) = self.client.installation_and_token(id).await?;
        Ok(token.expose_secret().to_string())
    }

    fn label_colour(&self, name: &str) -> &'static str {
        match name {
            "todo" => "fbca04",
            "bug" => "d73a4a",
            "deprecated" => "6a737d",
            "anko" => "672422",       
            _ => "ededed",           
        }
    }
    
    pub async fn get_issue(&self, id: InstallationId, owner: &str, repo: &str, issue_number: u64) -> Result<Option<ExistingIssue>, BoxError> {
        let repo_client = self.client.installation(id)?;
        
        match repo_client.issues(owner, repo).get(issue_number).await {
            Ok(issue) => {
                let existing = match issue.state {
                    IssueState::Open => ExistingIssue::Open(issue.number),
                    IssueState::Closed => ExistingIssue::Closed(issue.number),
                    _ => ExistingIssue::Open(issue.number),
                };
                Ok(Some(existing))
            }
            Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn ensure_label(&self, id: InstallationId, owner: &str, repo: &str, name: &str) -> Result<(), BoxError> {
        let repo_client = self.client.installation(id)?;

        match repo_client.issues(owner, repo).get_label(name).await {
            Ok(_) => Ok(()), // already exists
            Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => {
                repo_client
                    .issues(owner, repo)
                    .create_label(name, self.label_colour(name), "") 
                    .await?;
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn create_issue(&self, id: InstallationId, owner: &str, repo: &str, tag: &TrackedTag) -> Result<u64, BoxError> {
        let kind_label = Kind::to_string(&tag.kind);

        self.ensure_label(id, owner, repo, kind_label.as_str()).await?;
        self.ensure_label(id, owner, repo, BOT_LABEL).await?;

        let repo_client = self.client.installation(id)?;

        let body = format!(
            r#"## Source

            - File: `{}`
            - Kind: `{}`
            "#,
            tag.file.to_string_lossy().into_owned(),
            Kind::to_string(&tag.kind),
        );

        let mut labels = tag.labels.clone();
        labels.push(kind_label.to_string());
        labels.push(BOT_LABEL.to_string());

        let issue = repo_client
            .issues(owner, repo)
            .create(&tag.message)
            .body(body)
            .labels(labels)
            .send()
            .await?;

        Ok(issue.number)
    }   

    pub async fn close_issue(&self, id: InstallationId, owner: &str, repo: &str, issue_number: u64) -> Result<(), BoxError> {
        let repo_client = self.client.installation(id)?;
        repo_client
        .issues(owner, repo)
        .update(issue_number)
        .state(octocrab::models::IssueState::Closed)
        .send()
        .await?;
        
        Ok(())
    }
    
}