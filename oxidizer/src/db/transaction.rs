use async_trait::async_trait;
use quaint::{
    pooled::{PooledConnection, Quaint},
    prelude::*,
};

use crate::db::GenericClient;
use crate::error::*;

pub struct Transaction {
    pub(crate) client: PooledConnection,
}

impl Transaction {
    pub(crate) async fn from_pool(pool: &Quaint) -> DBResult<Transaction> {
        let client = pool.check_out().await?;

        client.execute_raw("BEGIN", &[]).await?;

        let transaction = Transaction { client };

        Ok(transaction)
    }

    pub async fn commit(self) -> DBResult<()> {
        self.client.execute_raw("COMMIT", &[]).await?;
        Ok(())
    }

    pub async fn rollback(self) -> DBResult<()> {
        self.client.execute_raw("ROLLBACK", &[]).await?;
        Ok(())
    }
}

#[async_trait]
impl GenericClient for Transaction {
    async fn execute(&self, query: &str, params: &'_ [Value<'_>]) -> Result<u64, Error> {
        let result = self.client.execute_raw(query, params).await?;

        Ok(result)
    }

    async fn query(&self, query: &str, params: &'_ [Value<'_>]) -> Result<ResultSet, Error> {
        let result = self.client.query_raw(query, params).await?;

        Ok(result)
    }
}
