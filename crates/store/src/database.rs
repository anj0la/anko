use firestore::{FirestoreDb, FirestoreDbOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ISSUE_COLLECTION_NAME: &'static str = "issues";

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Serialize, Deserialize)]
struct IssueDoc {
    owner: String,
    repo: String,
    hash: String,
    issue_number: u64,
}

pub struct Database {
    pub db: FirestoreDb,
}

impl Database {
    pub async fn new(gcp_project: &str) -> Result<Self, BoxError> {
        let db =
            FirestoreDb::with_options(FirestoreDbOptions::new(gcp_project.to_string())).await?;

        Ok(Self { db })
    }

    pub async fn get(&self, owner: &str, repo: &str, hash: &str) -> Result<Option<u64>, BoxError> {
        let doc: Option<IssueDoc> = self
            .db
            .fluent()
            .select()
            .by_id_in(ISSUE_COLLECTION_NAME)
            .obj()
            .one(format!("{owner}_{repo}_{hash}"))
            .await?;

        Ok(doc.map(|d| d.issue_number))
    }
    
    pub async fn list(&self, owner: &str, repo: &str) -> Result<HashMap<String, u64>, BoxError> {
        let docs: Vec<IssueDoc> = self
            .db
            .fluent()
            .select()
            .from(ISSUE_COLLECTION_NAME)
            .filter(|q| {
                q.for_all([
                    q.field("owner").eq(owner),
                    q.field("repo").eq(repo),
                ])
            })
            .obj()
            .query()
            .await?;

        Ok(docs.into_iter().map(|d| (d.hash, d.issue_number)).collect())
    }

    pub async fn set(
        &self,
        owner: &str,
        repo: &str,
        hash: &str,
        issue_number: u64,
    ) -> Result<(), BoxError> {
        let doc = IssueDoc {
            owner: owner.to_string(),
            repo: repo.to_string(),
            hash: hash.to_string(),
            issue_number,
        };

        self.db
            .fluent()
            .update()
            .in_col(ISSUE_COLLECTION_NAME)
            .document_id(format!("{owner}_{repo}_{hash}"))
            .object(&doc)
            .execute::<()>()
            .await?;

        Ok(())
    }

    pub async fn delete(&self, owner: &str, repo: &str, hash: &str) -> Result<(), BoxError> {
        self.db
            .fluent()
            .delete()
            .from(ISSUE_COLLECTION_NAME)
            .document_id(format!("{owner}_{repo}_{hash}"))
            .execute()
            .await?;

        Ok(())
    }
}