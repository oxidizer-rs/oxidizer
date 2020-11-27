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
//!     #[primary_key(increments)]
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
//! ### #[primary_key(increments)]
//! Required
//! Field attribute used to mark the field as the primary key.
//! `increments` will make the field integer autoincrement
//!
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! struct Entity {
//!     #[primary_key(increments)]
//!     id: i32
//! }
//! ```
//!
//! ```
//! use oxidizer::*;
//! #[derive(Entity)]
//! struct Entity {
//!     #[primary_key()]
//!     name: String
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
//!     #[primary_key(increments)]
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
//!     #[primary_key(increments)]
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
//!     #[primary_key(increments)]
//!     id: i32,
//!
//!     name: String,
//!     email: String,
//! }
//! ```
//!
//! ### #[field_ignore]
//! Ignores the specified field. The field type must implement the `Default` trait.
//!
//! ```
//! use oxidizer::*;
//! #[derive(Default, Entity)]
//! struct MyEntity {
//!     #[primary_key(increments)]
//!     id: i32,
//!
//!     name: String,
//!     #[field_ignore]
//!     email: String,
//! }
//! ```
//!
//! ### #[custom_type]
//! The custom type attribute lets you override the default type provided by oxidizer.
//!
//! ```
//! use oxidizer::*;
//! pub enum MyEnum {
//!     Item1,
//!     Item2,
//! }
//!
//! pub enum ConvertError {
//!     Error
//! }
//!
//! impl std::fmt::Display for ConvertError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.write_str("Error trying to convert")
//!     }
//! }
//!
//! impl TryFrom<&MyEnum> for i32 {
//!     type Error = ConvertError;
//!
//!     fn try_from(v: &MyEnum) -> Result<Self, Self::Error> {
//!         match v {
//!             MyEnum::Item1 => Ok(0),
//!             MyEnum::Item2 => Ok(1),
//!         }
//!     }
//! }
//!
//! impl TryFrom<i32> for MyEnum {
//!     type Error = ConvertError;
//!
//!     fn try_from(v: i32) -> Result<Self, Self::Error> {
//!         match v {
//!             0 => Ok(MyEnum::Item1),
//!             1 => Ok(MyEnum::Item2),
//!             _ => Err(ConvertError::Error),
//!         }
//!     }
//! }
//!
//! #[derive(Entity)]
//! pub struct TestCustomType {
//!     #[primary_key(increments)]
//!     id: i32,
//!
//!     #[custom_type(ty = "i32")]
//!     my_enum: MyEnum,
//! }
//! ```
//! The custom type requires you to explicity implement the related `TryFrom` trait functions to convert between the
//! actual type and the overriden type. The error type from the `TryFrom` trait must implement the `std::fmt::Display` trait
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
//!     #[primary_key(increments)]
//!     id: i32,
//! }
//!
//! #[derive(Entity)]
//! struct TestRelation {
//!     #[primary_key(increments)]
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
//!     #[primary_key(increments)]
//!     id: i32,
//!     name: String
//! }
//!
//! #[derive(Default, Entity)]
//! pub struct TargetEntity {
//!     #[primary_key(increments)]
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
//!     #[primary_key(increments)]
//!     id: i32,
//!     name: String
//! }
//!
//! #[derive(Default, Entity)]
//! #[has_many(model="Entity", field="entity_id", through="TestManyToMany")]
//! pub struct TargetEntity {
//!     #[primary_key(increments)]
//!     id: i32,
//! }
//!
//! #[derive(Default, Entity)]
//! pub struct TestManyToMany {
//!     #[primary_key(increments)]
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
pub use db::types::*;
pub use db::*;

pub mod entity;
pub use entity::*;

//pub mod migration;

/// Re-export of [async_trait::async_trait](https://crates.io/crates/async-trait)
pub use async_trait::async_trait;

pub use quaint::{
    connector::{ResultRow, ResultRowRef},
    Value,
};

pub use oxidizer_entity_macro::*;

#[cfg(test)]
mod tests_macro;

//#[cfg(test)]
//mod migrations;

pub use std::convert::TryFrom;
