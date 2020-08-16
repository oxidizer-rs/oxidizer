//! # Oxidizer
//! A simple orm based on [tokio-postgres](https://crates.io/crates/tokio-postgres) and [refinery](https://crates.io/crates/refinery)
//! ```ignore
//! #[async_trait]
//! pub trait Entity: Sized {
//!     async fn save(&mut self, db: &DB) -> DBResult<bool>;
//!     async fn delete(&mut self, db: &DB) -> DBResult<bool>;
//!
//!     fn from_row(row: &Row) -> Self;
//!     fn create_migration() -> DBResult<Migration>;
//!     fn get_table_name() -> String;
//!
//!     async fn find(db: &DB, query: &str, params: &'_ [&'_ (dyn ToSql + Sync)]) -> DBResult<Vec<Self>>;
//!     async fn first(db: &DB, query: &str, params: &'_ [&'_ (dyn ToSql + Sync)]) -> DBResult<Option<Self>>;
//! }
//! ```
//! ```
//! use oxidizer::*;
//! use chrono::{DateTime, Utc};
//!
//! #[derive(Entity)]
//! #[derive(Default)]
//! pub struct MyEntity {
//!     #[primary_key]
//!     id: i32,
//!
//!     name: String,
//!
//!     #[indexed]
//!     integer: i32,
//!     integer64: i64,
//!
//!     float: f32,
//!     double: f64,
//!
//!     boolean: bool,
//!
//!     datetime: Option<DateTime<Utc>>,
//! }
//!
//! #[tokio::test]
//! async fn test_my_entity() {
//!     let uri = "postgres://postgres:alkje2lkaj2e@db/postgres";
//!     let max_open = 50; // mobc
//!     let ca_file: Option<&str> = None;
//!     let db = DB::connect(&uri, max_open, ca_file).await.unwrap();
//!
//!     db.migrate_tables(&[MyEntity::create_migration().unwrap()]).await.unwrap();
//!
//!     let mut entity = MyEntity::default();
//!     let creating = entity.save(&db).await.unwrap();
//!     assert_eq!(creating, true);
//! }
//!
//! ```
//!
//!
//! ## Attributes
//!
//! Derive attributes can be used to create indexes, change the default table name and
//! create reverse relation accessors
//!
//! ### #[primary_key]
//! Required
//! Field attribute used to mark the field as the primary key, this will make the field autoincrement
//!
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! struct Entity {
//!     #[primary_key]
//!     id: i32
//! }
//! ```
//!
//! ### #[indexed]
//! Make the specified field indexed in the db
//!
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! struct Entity {
//!     #[primary_key]
//!     id: i32,
//!     #[indexed]
//!     name: String,
//! }
//! ```
//!
//! ### #[relation]
//! See [Relations](#Relations)
//!
//! ### #[has_many]
//! See [Relations](#Relations)
//!
//! ### #[entity]
//! General settings for the entity struct
//!
//! #### table_name: String;
//! Allows one to change the table name of the entity
//!
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! #[entity(table_name="custom_table_name")]
//! struct Entity {
//!     #[primary_key]
//!     id: i32
//! }
//! ```
//!
//! ### #[index]
//! Creates a custom index/constraint on one or more column
//!
//! ```
//! use oxidizer::*;
//! #[derive(Default, Entity)]
//! #[index(name="myindex", columns="name, email", unique)]
//! struct MyEntity {
//!     #[primary_key]
//!     id: i32,
//!
//!     name: String,
//!     email: String,
//! }
//! ```
//!
//!
//! ## Relations
//!
//! ### #[relation]
//! Relations can be created using the `relation` attribute as in the example:
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! struct Entity {
//!     #[primary_key]
//!     id: i32,
//! }
//!
//! #[derive(Entity)]
//! struct TestRelation {
//!     #[primary_key]
//!     id: i32,
//!     device_id: String,
//!
//!     #[relation(model="Entity", key="id")]
//!     entity_id: i32,
//! }
//! ```
//!
//! This will implement for `TestRelation` the following generated trait:
//! ```ignore
//! #[oxidizer::async_trait]
//! pub trait __AccessorTestRelationToEntity {
//!     async fn get_test_entity(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<Entity>;
//!     async fn set_test_entity(&mut self, db: &oxidizer::db::DB, v: &Entity) -> oxidizer::db::DBResult<()>;
//! }
//! ```
//!
//! #[has_many]
//! 1-to-many or many-to-many relations can be achieved using the `has_many` attribute
//!
//! ### basic (1-to-many)
//!
//! ```
//! use oxidizer::*;
//!
//! #[derive(Entity)]
//! #[derive(Default)]
//! #[has_many(model="TargetEntity", field="entity_id")]
//! pub struct Entity {
//!     #[primary_key]
//!     id: i32,
//!     name: String
//! }
//!
//! #[derive(Default, Entity)]
//! pub struct TargetEntity {
//!     #[primary_key]
//!     id: i32,

//!     #[relation(model="Entity", key="id")]
//!     entity_id: i32
//! }
//! ```
//! This will create helper functions to access all the `TargetEntity` that Entity has.
//! This is what the generated trait and implementation looks like (implementaion is also generated).
//!
//! ```ignore
//! #[oxidizer::async_trait]
//! pub trait __AccessorHasManyTargetEntityToEntity {
//!     async fn get_all_test_entity(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<Vec<Entity>>;
//! }
//! ```
//!
//! ### With a through table (many-to-many)
//! ```
//! use oxidizer::*;
//!
//! #[derive(Entity)]
//! #[derive(Default)]
//! pub struct Entity {
//!     #[primary_key]
//!     id: i32,
//!     name: String
//! }
//!
//! #[derive(Default, Entity)]
//! #[has_many(model="Entity", field="entity_id", through="TestManyToMany")]
//! pub struct TargetEntity {
//!     #[primary_key]
//!     id: i32,
//! }
//!
//! #[derive(Default, Entity)]
//! pub struct TestManyToMany {
//!     #[primary_key]
//!     id: i32,
//!
//!     #[relation(model="TargetEntity", key="id")]
//!     target_id: i32,
//!
//!     #[relation(model="Entity", key="id")]
//!     entity_id: i32,
//! }
//! ```
//! This will create helper functions to access the related entities. This is what the generated trait looks like (implementaion is also generated):
//! ```ignore
//! #[oxidizer::async_trait]
//! pub trait __AccessorHasManyTargetEntityToEntity {
//!     async fn get_all_test_entity(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<Vec<TestManyToMany>>;
//! }
//! ```
//!
//!

pub mod db;
pub use db::*;

pub mod entity;
pub use entity::*;

pub mod migration;

/// Re-export of [async_trait::async_trait](https://crates.io/crates/async-trait)
pub use async_trait::async_trait;
pub use tokio_postgres;
pub use tokio_postgres::types as db_types;

pub use barrel::types;

pub use oxidizer_entity_macro::*;

#[cfg(test)]
mod tests_macro;

#[cfg(test)]
mod migrations;
