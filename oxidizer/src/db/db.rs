// use async_trait::async_trait;
// use mobc::Manager;
// use mobc::Pool;
// use openssl::ssl::{SslConnector, SslMethod};
// use postgres_openssl::MakeTlsConnector;
// use refinery::{Report, Runner};
// use std::str::FromStr;

// use super::super::migration::Migration;
// use super::error::*;

// impl DB {
//     pub async fn create(
//         &self,
//         query: &str,
//         params: &'_ [&'_ (dyn ToSql + Sync)],
//     ) -> Result<u64, Error> {
//         self.execute(query, params).await
//     }

//     pub async fn migrate_tables(&self, ms: &[Migration]) -> Result<Report, Error> {
//         let ref_migrations: Vec<refinery::Migration> = ms
//             .as_ref()
//             .iter()
//             .enumerate()
//             .filter_map(|(i, m)| {
//                 let sql = m.raw.make::<Pg>();

//                 let name = format!("V{}__{}.rs", i, m.name);

//                 let migration = refinery::Migration::unapplied(&name, &sql).unwrap();

//                 Some(migration)
//             })
//             .collect();

//         let runner = refinery::Runner::new(&ref_migrations);

//         self.migrate(runner).await
//     }

//     pub async fn migrate(&self, runner: Runner) -> Result<Report, Error> {
//         let runner = runner.set_abort_divergent(false);
//         match &self.pool {
//             ConnectionPool::TLS(pool) => {
//                 let mut client = pool.get().await.map_err(|err| Error::MobcError(err))?;
//                 Ok(runner
//                     .run_async(&mut *client)
//                     .await
//                     .map_err(|err| Error::RefineryError(err))?)
//             }
//             ConnectionPool::NoTLS(pool) => {
//                 let mut client = pool.get().await.map_err(|err| Error::MobcError(err))?;
//                 Ok(runner
//                     .run_async(&mut *client)
//                     .await
//                     .map_err(|err| Error::RefineryError(err))?)
//             }
//         }
//     }
// }

use super::error::*;
use sqlx::any::AnyArguments;
use sqlx::any::AnyPool;
use sqlx::any::AnyPoolOptions;
use sqlx::any::AnyRow;
use sqlx::Done;

pub struct DB {
    pool: AnyPool,
}

impl DB {
    pub async fn connect(uri: &str, max_open: u32, ca_file: Option<&str>) -> Result<Self, Error> {
        let pool = AnyPoolOptions::new()
            .max_connections(max_open)
            .connect(uri)
            .await?;

        Ok(DB { pool })
    }

    pub async fn execute<'a, Q: Into<&'a str>>(
        &self,
        query: Q,
        arguments: AnyArguments<'a>,
    ) -> Result<u64, Error> {
        let rows_affected = sqlx::query_with(query.into(), arguments)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(rows_affected)
    }

    pub async fn query<'a, Q: Into<&'a str>>(
        &self,
        query: Q,
        arguments: AnyArguments<'a>,
    ) -> Result<Vec<AnyRow>, Error> {
        let rows = sqlx::query_with(query.into(), arguments)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}

