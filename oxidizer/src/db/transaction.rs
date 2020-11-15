use async_trait::async_trait;
use std::future::Future;
use tokio_postgres::Transaction as PgTransaction;

use super::*;

pub struct Transaction<'a> {
    internal: PgTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub(crate) async fn new(
        client: &'a mut mobc::Connection<connections::ConnectionManager>,
    ) -> Result<Transaction<'a>, Error> {
        let internal = client
            .transaction()
            .await
            .map_err(|err| Error::PostgresError(err))?;

        Ok(Transaction { internal })
    }

    pub async fn commit(self) -> Result<(), Error> {
        self.internal
            .commit()
            .await
            .map_err(|err| Error::PostgresError(err))
    }

    pub async fn rollback(self) -> Result<(), Error> {
        self.internal
            .rollback()
            .await
            .map_err(|err| Error::PostgresError(err))
    }
}

#[async_trait]
impl GenericClient for Transaction<'_> {
    async fn execute(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, Error> {
        let insert = self
            .internal
            .prepare(query)
            .await
            .map_err(Error::PostgresError)?;

        self.internal
            .execute(&insert, params)
            .await
            .map_err(Error::PostgresError)
    }

    async fn query(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        let insert = self
            .internal
            .prepare(query)
            .await
            .map_err(Error::PostgresError)?;

        self.internal
            .query(&insert, params)
            .await
            .map_err(Error::PostgresError)
    }
}
