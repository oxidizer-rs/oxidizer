use tokio_postgres::Row;

use super::async_trait;
use super::db::{DBResult, DB};
use super::db_types::ToSql;
use super::migration::Migration;
use super::GenericClient;

/// Trait implemented by all derived Entitities
#[async_trait]
pub trait IEntity: Sized {
    async fn save(&mut self, db: &impl GenericClient) -> DBResult<bool>;
    async fn delete(&mut self, db: &impl GenericClient) -> DBResult<bool>;

    fn is_synced_with_db(&self) -> bool;

    fn from_row(row: &Row) -> DBResult<Self>;
    fn create_migration() -> DBResult<Migration>;
    fn get_table_name() -> String;

    async fn find(
        db: &impl GenericClient,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> DBResult<Vec<Self>>;
    async fn first(
        db: &impl GenericClient,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> DBResult<Option<Self>>;
}
