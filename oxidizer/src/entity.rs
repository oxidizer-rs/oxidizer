use quaint::prelude::*;

use crate::async_trait;
use crate::db::{DBResult, DB};
use crate::ResultRow;

/// Trait implemented by all derived Entitities
#[async_trait]
pub trait IEntity: Sized {
    async fn save(&mut self, db: &DB) -> DBResult<bool>;
    async fn delete(&mut self, db: &DB) -> DBResult<bool>;

    fn is_synced_with_db(&self) -> bool;

    fn from_row(row: ResultRow) -> DBResult<Self>;
    fn get_table_name() -> String;

    async fn find(db: &DB, query: &str, params: &'_ [Value<'_>]) -> DBResult<Vec<Self>>;
    async fn first(db: &DB, query: &str, params: &'_ [Value<'_>]) -> DBResult<Option<Self>>;
}
