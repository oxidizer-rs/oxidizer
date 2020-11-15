use async_trait::async_trait;
use barrel::backend::Pg;
use mobc::Pool;
use refinery::{Report, Runner};
use std::future::Future;
use std::str::FromStr;
use tokio_postgres::{row::Row, types::ToSql, Client};

use super::super::migration::Migration;
use super::connections::{self, *};
use super::error::*;
use super::generic_client::GenericClient;
use super::transaction::Transaction;

#[derive(Clone)]
pub struct DB {
    pool: Pool<ConnectionManager>,
}

unsafe impl std::marker::Sync for DB {}

impl DB {
    pub async fn connect(uri: &str, max_open: u64, ca_file: Option<&str>) -> Result<Self, Error> {
        let config = tokio_postgres::Config::from_str(uri).map_err(Error::PostgresError)?;

        let provider = connections::create_connection_provider(config, ca_file).await?;
        let manager = ConnectionManager { provider };

        Ok(DB {
            pool: Pool::builder().max_open(max_open).build(manager),
        })
    }

    pub async fn create(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> Result<u64, Error> {
        self.execute(query, params).await
    }

    pub async fn with_transaction<'a, T, F, Fut>(&self, cb: F) -> Result<T, Error>
    where
        F: FnOnce(&Transaction) -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        let mut client = self.pool.get().await.map_err(Error::MobcError)?;

        let transaction = Transaction::new(&mut client).await?;

        let res = cb(&transaction).await;

        match res {
            Ok(r) => {
                transaction.commit().await?;
                Ok(r)
            }
            Err(err) => {
                transaction.rollback().await?;
                Err(err)
            }
        }
    }

    pub async fn migrate_tables(&self, ms: &[Migration]) -> Result<Report, Error> {
        let ref_migrations: Vec<refinery::Migration> = ms
            .as_ref()
            .iter()
            .enumerate()
            .filter_map(|(i, m)| {
                let sql = m.raw.make::<Pg>();

                let name = format!("V{}__{}.rs", i, m.name);

                let migration = refinery::Migration::unapplied(&name, &sql).unwrap();

                Some(migration)
            })
            .collect();

        let runner = refinery::Runner::new(&ref_migrations);

        self.migrate(runner).await
    }

    pub async fn migrate(&self, runner: Runner) -> Result<Report, Error> {
        let runner = runner.set_abort_divergent(false);
        let mut client = self.pool.get().await.map_err(Error::MobcError)?;
        Ok(runner
            .run_async(&mut *client)
            .await
            .map_err(Error::RefineryError)?)
    }
}

#[async_trait]
impl GenericClient for DB {
    async fn execute(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> Result<u64, Error> {
        let client = self.pool.get().await.map_err(Error::MobcError)?;

        let insert = client.prepare(query).await.map_err(Error::PostgresError)?;

        client
            .execute(&insert, params)
            .await
            .map_err(Error::PostgresError)
    }

    async fn query(
        &self,
        query: &str,
        params: &'_ [&'_ (dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error> {
        let client = self.pool.get().await.map_err(Error::MobcError)?;

        let insert = client.prepare(query).await.map_err(Error::PostgresError)?;

        client
            .query(&insert, params)
            .await
            .map_err(Error::PostgresError)
    }
}
