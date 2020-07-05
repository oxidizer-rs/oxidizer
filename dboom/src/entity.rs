use tokio_postgres::Row;
use barrel::Migration;

use super::async_trait;
use super::db::{DBResult, DB};

#[async_trait]
pub trait Entity {
    async fn save(&mut self, db: &DB) -> DBResult<bool>;
    async fn delete(&self, db: &DB) -> DBResult<bool>;

    async fn from_row(row: &Row) -> Self;
    async fn create_migration() -> DBResult<Migration>;
}