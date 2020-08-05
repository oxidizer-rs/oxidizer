mod db;
mod entity;
mod migration;
pub use migration::*;

/// Re-export of [async_trait::async_trait](https://crates.io/crates/async-trait)
pub use async_trait::async_trait;
pub(crate) use tokio_postgres;
pub use tokio_postgres::types as db_types;

pub use barrel::{types};
pub use refinery::include_migration_mods;

pub use oxidizer_entity_macro::*;

#[cfg(test)]
mod tests_macro;

#[cfg(test)]
mod migrations;