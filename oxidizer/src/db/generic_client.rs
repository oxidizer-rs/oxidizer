use async_trait::async_trait;
use tokio_postgres::{row::Row, types::ToSql};

use super::Error;

#[async_trait]
pub trait GenericClient: Sync {
    async fn execute(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> Result<u64, Error>;

    async fn query(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error>;
}
