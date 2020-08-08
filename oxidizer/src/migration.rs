//!
//! # Migrations
//!
//! Migrations are handled by [refinery](https://crates.io/crates/refinery) and barrel. Go ahead and read their docs.
//! The derive macro generates the migration code
//! for each entity automatically. The only thing needed is include each Entity's migration in a file (module)
//! A macro to generate the necessary code for each migration module is provided.
//! NOTE: Please take note that this is highly experimental and it can change frequently.
//!
//! ```ignore
//! + src/
//! +-- entities/
//! +--+-- mod.rs
//! +--+-- person.rs
//! +--+-- account.rs
//! +--migrations/
//! +--+-- mod.rs
//! +--+-- V00001__person.rs
//! +--+-- V00002__account.rs
//! ```
//!
//!
//! - V00001__person.rs
//!
//! ```ignore
//! use oxidizer::create_migration_module;
//! use oxidizer::entity::IEntity;
//!
//! use crate::entities::Person;
//!
//! create_migration_module!(Person);
//! ```
//! - migrations/mod.rs
//!
//! ```ignore
//! use oxidizer::include_migration_mods;
//!
//! include_migration_mods!();
//!
//! ```
//!
//! With the correct file struct you can now create a runner and apply migrations with:
//!
//! ```
//! use oxidizer::*;
//! #[tokio::test]
//! async fn test_migrate() {
//!  let runner = crate::migrations::runner();
//!
//!  let uri = "postgres://postgres:alkje2lkaj2e@db/postgres";
//!  let max_open = 50; // mobc
//!  let ca_file: Option<&str> = None;
//!  let db = DB::connect(&uri, max_open, ca_file).await.unwrap();
//!  db.migrate(runner).await.unwrap();
//! }
//! ```
//!

use barrel::{backend::Pg, Migration as RawMigration};

/// Migration abstract layer
pub struct Migration {
    pub name: String,

    pub raw: RawMigration,
}

impl Migration {
    /// Creates a new migration
    pub fn new(name: &str) -> Self {
        Migration{
            name: name.to_string(),

            raw: RawMigration::new(),
        }
    }

    /// Builds the raw query from the migration
    pub fn make(&self) -> String {
        self.raw.make::<Pg>()
    }
}


/// Creates a new migration module
#[macro_export]
macro_rules! create_migration_module {
    ($entity:ident) => {

        pub fn migration() -> String {
            let m = <$entity>::create_migration().expect(concat!("Could not create migration for ", stringify!($entity)));
            m.make()
        }

    };
}