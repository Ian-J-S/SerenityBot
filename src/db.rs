use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub immuwune: HashSet<String>,
}

impl Database {
    pub async fn load(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if Path::new(path).exists() {
            let contents = fs::read_to_string(path).await?;
            println!("Successfully loaded db file");
            Ok(serde_json::from_str(&contents)?)
        } else {
            println!("No db file found, created new db");
            Ok(Self::default())
        }
    }

    pub async fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }
}
