use async_trait::async_trait;
use quaint::prelude::*;

use crate::Error;

#[async_trait]
pub trait GenericClient {
    async fn execute(&self, query: &str, params: &'_ [Value<'_>]) -> Result<u64, Error>;
    async fn query(&self, query: &str, params: &'_ [Value<'_>]) -> Result<ResultSet, Error>;
}
