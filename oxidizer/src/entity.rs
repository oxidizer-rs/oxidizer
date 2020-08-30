use tokio_postgres::Row;

use super::async_trait;
use super::db::{DBResult, DB};
use super::db_types::ToSql;
use super::migration::Migration;

/// Trait implemented by all derived Entitities
#[async_trait]
pub trait IEntity: Sized {
    async fn save(&mut self, db: &DB) -> DBResult<bool>;
    async fn delete(&mut self, db: &DB) -> DBResult<bool>;

    fn is_synced_with_db(&self) -> bool;

    fn from_row(row: &Row) -> DBResult<Self>;
    fn create_migration() -> DBResult<Migration>;
    fn get_table_name() -> String;

    async fn find(
        db: &DB,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> DBResult<Vec<Self>>;
    async fn first(
        db: &DB,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> DBResult<Option<Self>>;
}
