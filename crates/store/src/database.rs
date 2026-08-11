use firestore::{FirestoreDb, FirestoreDbOptions};
use serde::{Deserialize, Serialize};

const ISSUE_COLLECTION_NAME: &'static str = "issues";

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Serialize, Deserialize)]
struct IssueDoc {
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

    pub async fn set(
        &self,
        owner: &str,
        repo: &str,
        hash: &str,
        issue_number: u64,
    ) -> Result<(), BoxError> {
        let doc = IssueDoc { issue_number };

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
}
