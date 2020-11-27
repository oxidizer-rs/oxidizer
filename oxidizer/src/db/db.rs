use quaint::{pooled::Quaint, prelude::*};

use super::Error;

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

    pub async fn execute(&self, query: &str, params: &'_ [Value<'_>]) -> Result<u64, Error> {
        let client = self.pool.check_out().await?;

        let result = client.execute_raw(query, params).await?;

        Ok(result)
    }

    pub async fn query(&self, query: &str, params: &'_ [Value<'_>]) -> Result<ResultSet, Error> {
        let client = self.pool.check_out().await?;

        let result = client.query_raw(query, params).await?;

        Ok(result)
    }

    //pub async fn migrate_tables(&self, ms: &[Migration]) -> Result<Report, Error> {
    //let ref_migrations: Vec<refinery::Migration> = ms
    //.as_ref()
    //.iter()
    //.enumerate()
    //.filter_map(|(i, m)| {
    //let sql = m.raw.make::<Pg>();

    //let name = format!("V{}__{}.rs", i, m.name);

    //let migration = refinery::Migration::unapplied(&name, &sql).unwrap();

    //Some(migration)
    //})
    //.collect();

    //let runner = refinery::Runner::new(&ref_migrations);

    //self.migrate(runner).await
    //}

    //pub async fn migrate(&self, runner: Runner) -> Result<Report, Error> {
    //let runner = runner.set_abort_divergent(false);
    //let mut client = self.pool.get().await.map_err(Error::MobcError)?;
    //Ok(runner
    //.run_async(&mut *client)
    //.await
    //.map_err(Error::RefineryError)?)
    //}
}
