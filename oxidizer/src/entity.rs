use sqlx::any::AnyArguments;
use sqlx::any::AnyRow;

use super::async_trait;
use super::db::{DBResult, DB};
use super::migration::Migration;

/// Trait implemented by all derived Entitities
#[async_trait]
pub trait IEntity: Sized {
    async fn save(&mut self, db: &DB) -> DBResult<bool>;
    async fn delete(&mut self, db: &DB) -> DBResult<bool>;

    fn is_synced_with_db(&self) -> bool;

    fn from_row(row: &AnyRow) -> DBResult<Self>;
    fn create_migration() -> DBResult<Migration>;
    fn get_table_name() -> String;

    async fn find<'a>(
        db: &DB,
        query: &str,
        arguments: AnyArguments<'a>,
    ) -> DBResult<Vec<Self>>;
    async fn first<'a>(
        db: &DB,
        query: &str,
        arguments: AnyArguments<'a>,
    ) -> DBResult<Option<Self>>;
}
