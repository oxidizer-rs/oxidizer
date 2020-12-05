//!
//! # Migrations
//!
//!
//! ```ignore
//! + src/
//! +-- entities/
//! +--+-- mod.rs
//! +--+-- person.rs
//! +--+-- account.rs
//! +--migrations/
//! +--+-- V00001__person.sql
//! +--+-- V00002__account.sql
//! ```
//!
//!
//! - V00001__person.sql
//!
//! With the correct file struct you can now create a runner and apply migrations with:
//!
//! ```
//! use oxidizer::*;
//! #[tokio::test]
//! async fn test_migrate() {
//!  let runner = crate::migrations::runner();
//!
//!  let uri = "postgres://postgres:alkje2lkaj2e@db/postgres";
//!  let max_open = 50; // mobc
//!  let ca_file: Option<&str> = None;
//!  let db = DB::connect(&uri, max_open, ca_file).await.unwrap();
//!  db.migrate(runner).await.unwrap();
//! }
//! ```
//!

use hex;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio;

use crate::error::DBResult;

pub mod errors;
pub use errors::*;

pub mod runner;
pub use runner::*;

#[cfg(test)]
mod tests;

/// Migration abstract layer
pub struct Migration {
    pub filename: PathBuf,
    pub contents: String,
    pub hash: String,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Migration {
    async fn hash_file(path: &PathBuf) -> DBResult<(String, String)> {
        let contents = tokio::fs::read_to_string(path).await?;

        Ok(tokio::task::spawn_blocking(move || {
            let mut hasher = Sha256::new();
            hasher.update(contents.as_bytes());
            let result = hasher.finalize();
            (contents, hex::encode(result))
        })
        .await?)
    }

    /// Creates a migration from a file path
    pub async fn from_path(path: impl Into<PathBuf>) -> DBResult<Migration> {
        let path: PathBuf = path.into();

        if !path.exists() {
            return Err(MigrationError::FileDoesNotExist(path.to_str().unwrap().into()).into());
        }

        let (contents, hash) = Migration::hash_file(&path).await?;

        Ok(Migration {
            filename: path,
            hash,
            contents,
            executed_at: None,
        })
    }
}
