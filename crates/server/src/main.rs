use axum::Router;
use dotenvy::dotenv;
use std::sync::Arc;

use github;
use store;
mod handler;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    dotenv().ok(); // load variables from .env into system env
    // will change to load from Secret Manager probs

    let webhook_secret = std::env::var("WEBHOOK_SECRET")?;
    let gcp_project = std::env::var("GCP_PROJECT_ID")?;
    let app_id: u64 = std::env::var("GITHUB_APP_ID")?.parse()?;
    let private_key_path = std::env::var("GITHUB_APP_PRIVATE_KEY_PATH")?;

    let db = store::Database::new(&gcp_project).await?;
    let github = github::GitHubApp::new(app_id, &private_key_path)?;

    let state = handler::AppState {
        webhook_secret,
        db: Arc::new(db),
        github: Arc::new(github),
    };

    let app = Router::new()
        .route("/webhook", axum::routing::post(handler::webhook_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("listening on :8080");
    axum::serve(listener, app).await?;

    Ok(())
}
