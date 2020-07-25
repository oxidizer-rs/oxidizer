use async_trait::async_trait;
use mobc::Manager;
use mobc::Pool;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use std::str::FromStr;
use refinery::{Runner, Report};

use super::super::Migration;

use tokio_postgres::{
    tls::{MakeTlsConnect, TlsConnect},
    types::ToSql,
    row::Row,
    Client, Config, NoTls, Socket,
};
use barrel::{backend::Pg};

pub struct ConnectionManager<Tls> {
    config: Config,
    tls: Tls,
}

impl<Tls> ConnectionManager<Tls> {
    pub fn new(config: Config, tls: Tls) -> Self {
        Self { config, tls }
    }
}

#[async_trait]
impl<Tls> Manager for ConnectionManager<Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type Connection = Client;
    type Error = tokio_postgres::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tls = self.tls.clone();
        let (client, conn) = self.config.connect(tls).await?;
        mobc::spawn(conn);
        Ok(client)
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        conn.simple_query("").await?;
        Ok(conn)
    }
}


#[derive(Debug)]
pub enum Error {
    PostgresError(tokio_postgres::Error),
    OpensslError(openssl::error::ErrorStack),
    MobcError(mobc::Error<tokio_postgres::Error>),
    RefineryError(refinery::Error),
    DoesNotExist,
    ReferencedModelIsNotInDB,
    Other,
}

pub type DBResult<T> = std::result::Result<T, Error>;

#[derive(Clone)]
enum ConnectionPool {
    TLS(Pool<ConnectionManager<MakeTlsConnector>>),
    NoTLS(Pool<ConnectionManager<NoTls>>),
}

#[derive(Clone)]
pub struct DB {
    pool: ConnectionPool,
}

impl DB {
    pub async fn connect(uri: &str, max_open: u64, ca_file: Option<&str>) -> Result<Self, Error> {
            if let Some(ca_file) = ca_file {
            let mut builder =
                SslConnector::builder(SslMethod::tls()).map_err(|err| Error::OpensslError(err))?;

            builder
                .set_ca_file(ca_file)
                .map_err(|err| Error::OpensslError(err))?;

            let connector = MakeTlsConnector::new(builder.build());
            let config =
                tokio_postgres::Config::from_str(uri).map_err(|err| Error::PostgresError(err))?;
            let manager = ConnectionManager::new(config, connector);

            Ok(DB {
                pool: ConnectionPool::TLS(Pool::builder().max_open(max_open).build(manager)),
            })
        } else {
            let config =
                tokio_postgres::Config::from_str(uri).map_err(|err| Error::PostgresError(err))?;

                let manager = ConnectionManager::new(config, NoTls);

                Ok(DB {
                    pool: ConnectionPool::NoTLS(Pool::builder().max_open(max_open).build(manager)),
                })

            }


        }

    pub async fn create(&self, query: &str, params: &'_ [&'_ (dyn ToSql + Sync)]) -> Result<u64, Error> {
        self.execute(query, params).await
    }

    pub async fn execute(&self, query: &str, params: &'_ [&'_ (dyn ToSql + Sync)]) -> Result<u64, Error> {
        match &self.pool {
            ConnectionPool::TLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                let insert = client
                    .prepare(query)
                    .await
                    .map_err(|err| Error::PostgresError(err))?;

                client
                    .execute(&insert, params)
                    .await
                    .map_err(|err| Error::PostgresError(err))

            },
            ConnectionPool::NoTLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                let insert = client
                    .prepare(query)
                    .await
                    .map_err(|err| Error::PostgresError(err))?;

                client
                    .execute(&insert, params)
                    .await
                    .map_err(|err| Error::PostgresError(err))
            }
        }
    }

    async fn execute_raw(&self, sql: &str) -> Result<(), Error> {
        let _res = match &self.pool {
            ConnectionPool::TLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                client
                    .simple_query(sql)
                    .await
                    .map_err(|err| Error::PostgresError(err))

            },
            ConnectionPool::NoTLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                client
                    .simple_query(&sql)
                    .await
                    .map_err(|err| Error::PostgresError(err))
            }
        }?;

        Ok(())
    }

    pub async fn query(&self, query: &str, params: &'_ [&'_ (dyn ToSql + Sync)]) -> Result<Vec<Row>, Error> {
        match &self.pool {
            ConnectionPool::TLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                let insert = client
                    .prepare(query)
                    .await
                    .map_err(|err| Error::PostgresError(err))?;

                client
                    .query(&insert, params)
                    .await
                    .map_err(|err| Error::PostgresError(err))

            },
            ConnectionPool::NoTLS(pool) => {
                let client = pool.get().await.map_err(|err| Error::MobcError(err))?;

                let insert = client
                    .prepare(query)
                    .await
                    .map_err(|err| Error::PostgresError(err))?;

                client
                    .query(&insert, params)
                    .await
                    .map_err(|err| Error::PostgresError(err))
            }
        }
    }

    pub async fn migrate_tables(&self, ms: &[Migration]) -> Result<Report, Error> {
        let ref_migrations: Vec<refinery::Migration> = ms.as_ref().iter().enumerate().filter_map(|(i, m)| {
            let sql = m.raw.make::<Pg>();

            let name = format!("V{}__{}.rs", i, m.name);

            let migration = refinery::Migration::unapplied(&name, &sql).unwrap();

            Some(migration)
        }).collect();

        let runner = refinery::Runner::new(&ref_migrations).set_abort_divergent(false);

        self.migrate(&runner).await
    }

    pub async fn migrate(&self, runner: &Runner) -> Result<Report, Error> {
        match &self.pool {
            ConnectionPool::TLS(pool) => {
                let mut client = pool.get().await.map_err(|err| Error::MobcError(err))?;
                Ok(runner.run_async(&mut *client).await.map_err(|err| Error::RefineryError(err))?)
            },
            ConnectionPool::NoTLS(pool) => {
                let mut client = pool.get().await.map_err(|err| Error::MobcError(err))?;
                Ok(runner.run_async(&mut *client).await.map_err(|err| Error::RefineryError(err))?)
            }
        }
    }

}