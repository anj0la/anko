use firestore::{FirestoreDb, FirestoreDbOptions};
use serde::{Deserialize, Serialize};

  const ISSUE_COLLECTION_NAME: &'static str = "issues";

#[derive(Serialize, Deserialize)]
struct IssueDoc {
    issue_number: u64,
}

pub struct Database {
    pub db: FirestoreDb,
}

impl Database {
    pub async fn new(project_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = FirestoreDb::with_options(
        FirestoreDbOptions::new(project_id.to_string())
        ).await?;
        
         Ok(Self { db })
    }

    pub async fn get(&self, owner: &str, repo: &str, hash: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let doc: Option<IssueDoc> = self.db.fluent()
            .select()
            .by_id_in(ISSUE_COLLECTION_NAME)
            .obj()
            .one(format!("{owner}/{repo}#{hash}"))
            .await?;

        Ok(doc.map(|d| d.issue_number))
    }

    pub async fn set(&self, owner: &str, repo: &str, hash: &str, issue_number: u64) -> Result<(), Box<dyn std::error::Error>> {
        let doc = IssueDoc { issue_number };
        
        self.db.fluent()
            .update()
            .in_col(ISSUE_COLLECTION_NAME)
            .document_id(format!("{owner}/{repo}#{hash}"))
            .object(&doc)
            .execute::<()>()
            .await?;

        Ok(())
    }

}

