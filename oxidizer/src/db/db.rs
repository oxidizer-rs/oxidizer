use async_trait::async_trait;
use quaint::{pooled::Quaint, prelude::*};
use std::future::Future;

use crate::transaction::Transaction;
use crate::Error;
use crate::GenericClient;

#[derive(Clone)]
pub struct DB {
    pool: Quaint,
}

unsafe impl std::marker::Sync for DB {}

impl DB {
    pub async fn connect(uri: &str, max_open: usize, ca_file: Option<&str>) -> Result<Self, Error> {
        let mut builder = Quaint::builder(uri)?;
        builder.connection_limit(max_open);
        let pool = builder.build();

        Ok(DB { pool })
    }

    pub async fn create_transaction(&self) -> Result<Transaction, Error> {
        let transaction = Transaction::from_pool(&self.pool).await?;

        Ok(transaction)
    }

    pub async fn run_transaction<F, Fut, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Transaction) -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        let transaction = self.create_transaction().await?;

        match f(&transaction).await {
            Ok(t) => {
                transaction.commit().await?;
                Ok(t)
            }
            Err(err) => {
                transaction.rollback().await?;
                Err(err)
            }
        }
    }
}

#[async_trait]
impl GenericClient for DB {
    async fn execute(&self, query: &str, params: &'_ [Value<'_>]) -> Result<u64, Error> {
        let client = self.pool.check_out().await?;

        let result = client.execute_raw(query, params).await?;

        Ok(result)
    }

    async fn query(&self, query: &str, params: &'_ [Value<'_>]) -> Result<ResultSet, Error> {
        let client = self.pool.check_out().await?;

        let result = client.query_raw(query, params).await?;

        Ok(result)
    }
}
