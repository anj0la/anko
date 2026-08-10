use octocrab::{Octocrab, models::AppId, models::InstallationId, models::IssueState};
use jsonwebtoken::EncodingKey;
use std::env;
use std::fs;
use dotenvy::dotenv;

use scanner::{diff::ExistingIssue, parse::{TrackedTag, Kind}};

const BOT_LABEL: &str = "anko";

pub struct App {
    pub client: Octocrab,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok(); // load variables from .env into system env
        // will change to load from Secret Manager probs

        let app_secret = env::var("GITHUB_APP_ID").expect("GITHUB_APP_ID must be set!");
        // let webhook_secret = env::var("WEBHOOK_SECRET").expect("WEBHOOK_SECRET must be set!");
        let key_secret = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set!");

        // setting up the app settings
        let app_id = AppId(app_secret.parse()?);
        let key_contents = fs::read_to_string(key_secret)?;
        let encoding_key = EncodingKey::from_rsa_pem(key_contents.as_bytes())?;

        // installing app
        let app_client = Octocrab::builder()
            .app(app_id, encoding_key)
            .build()?;

        Ok(Self {
            client: app_client,
        })
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
    
    pub async fn get_issue(&self, id: InstallationId, owner: &str, repo: &str, issue_number: u64) -> Result<Option<ExistingIssue>, Box<dyn std::error::Error>> {
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

    async fn ensure_label(&self, id: InstallationId, owner: &str, repo: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo_client = self.client.installation(id)?;

        match repo_client.issues(owner, repo).get_label(name).await {
            Ok(_) => Ok(()), // already exists
            Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => {
                repo_client
                    .issues(owner, repo)
                    .create_label(name, self.label_colour(name), "") // could update to allow colours, but nah
                    .await?;
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn create_issue(&self, id: InstallationId, owner: &str, repo: &str, tag: &TrackedTag) -> Result<u64, Box<dyn std::error::Error>> {
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
            .assignees(vec![owner.to_string()])
            .send()
            .await?;

        Ok(issue.number)
    }   

    pub async fn close_issue(&self, id: InstallationId, owner: &str, repo: &str, issue_number: u64) -> Result<(), Box<dyn std::error::Error>> {
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
