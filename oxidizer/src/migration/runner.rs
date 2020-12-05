use log::*;
use std::path::PathBuf;

use crate::db::{types::*, *};
use crate::error::*;
use crate::migration::Migration;

#[derive(Debug)]
pub enum RunnerError {
    SQLError(String),
    InvalidHash(String),
    DirectoryNotExist(String),
    MigrationSQLError(String, String),
}

/// Migration runner. Used to manage migrations and execute the missing ones
pub struct Runner {
    directory: PathBuf,

    pub(crate) migrations: Vec<Migration>,
}

static MIGRATION_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS oxidizer_migrations (
   filename VARCHAR(100) NOT NULL,
   hash VARCHAR(64) NOT NULL,
   executed_at TIMESTAMP WITH TIME ZONE,
   PRIMARY KEY (filename)
)
"#;

impl Runner {
    /// creates a runner from a directory of migration files. All files must be named *.sql,
    /// otherwise they will be skipped
    pub async fn from_directory(path: impl Into<PathBuf>) -> DBResult<Runner> {
        let directory: PathBuf = path.into();

        if !directory.exists() {
            return Err(RunnerError::DirectoryNotExist(directory.to_str().unwrap().into()).into());
        }

        let mut migrations: Vec<Migration> = Vec::new();

        let mut iterator = tokio::fs::read_dir(&directory).await?;
        while let Some(entry) = iterator.next_entry().await? {
            match entry.file_name().to_str() {
                Some(s) => {
                    if !s.ends_with(".sql") {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            };

            debug!(
                "Found migration file: {}",
                entry.path().to_str().unwrap().to_string()
            );
            migrations.push(Migration::from_path(entry.path()).await?);
        }

        Ok(Runner {
            directory,
            migrations,
        })
    }

    pub(crate) async fn assert_migrations_table(&self, client: &DB) -> DBResult<()> {
        info!("Executing migration table checks query");
        client.execute(MIGRATION_TABLE_SQL, &[]).await?;

        Ok(())
    }

    pub(crate) async fn execute_single_migration(
        &self,
        client: &DB,
        migration: &Migration,
    ) -> DBResult<bool> {
        let result = client
            .query(
                "SELECT * FROM oxidizer_migrations WHERE filename = $1 LIMIT 1",
                &[migration.filename.to_str().unwrap().to_compatible_type()],
            )
            .await?;

        if let Some(row) = result.first() {
            let hash: String = row.get("hash").extract()?;

            if hash != migration.hash {
                return Err(
                    RunnerError::InvalidHash(migration.filename.to_str().unwrap().into()).into(),
                );
            }

            // migration already executed and the hashes are the same, skip this one
            return Ok(false);
        }

        debug!(
            "Executing migration: {}",
            migration.filename.to_str().unwrap()
        );
        client
            .execute(migration.contents.as_str(), &[])
            .await
            .map_err(|err| {
                RunnerError::MigrationSQLError(
                    migration.filename.to_str().unwrap().into(),
                    format!("{:?}", err),
                )
            })?;

        let _ = client
            .execute(
                "INSERT INTO oxidizer_migrations (filename, hash, executed_at) VALUES ($1, $2, $3)",
                &[
                    migration.filename.to_str().unwrap().to_compatible_type(),
                    migration.hash.as_str().to_compatible_type(),
                    chrono::Utc::now().to_compatible_type(),
                ],
            )
            .await?;

        Ok(true)
    }

    /// Execute the migrations loaded by this runner and return the number of successfully executed
    /// migrations
    pub async fn execute(&self, client: &DB) -> DBResult<usize> {
        self.assert_migrations_table(client).await?;

        let mut counter: usize = 0;
        for migration in self.migrations.iter() {
            if self.execute_single_migration(client, migration).await? {
                counter += 1;
            }
        }

        Ok(counter)
    }
}
