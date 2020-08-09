# Oxidizer ORM

A simple orm based on [tokio-postgres](https://crates.io/crates/tokio-postgres) and [refinery](https://crates.io/crates/refinery)

-------------------------
[![Actions Status][ci-badge]][ci-url]
[![Crates.io][crates-badge]][crates-url]
[![API Docs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[ci-badge]: https://github.com/oxidizer-rs/oxidizer/workflows/test/badge.svg
[ci-url]: https://github.com/oxidizer-rs/oxidizer/actions
[crates-badge]: https://img.shields.io/crates/v/oxidizer.svg
[crates-url]: https://crates.io/crates/oxidizer
[docs-badge]: https://docs.rs/oxidizer/badge.svg
[docs-url]: https://docs.rs/oxidizer
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/oxidizer-rs/oxidizer/blob/main/LICENSE

Oxidizer helps you reduce the boiler plate of writing entities, tables & migrations when using tokio-postgres and refinery.

⚠ Note that, while functional and working, this is in early stages. Use with caution. ⚠

Examples:

```rust
#[derive(Entity)]
#[derive(Default)]
pub struct MyEntity {
    #[primary_key]
    id: i32,

    name: String,

    #[indexed]
    integer: i32,
    integer64: i64,

    float: f32,
    double: f64,

    boolean: bool,

    datetime: Option<DateTime<Utc>>,
}
```

```rust
#[derive(Entity)]
#[entity(table_name="custom2")]
#[index(name="myindex", columns="name, datetime", unique)]
#[has_many(model="TestReverseRelation", field="entity_id")]
#[has_many(model="TestEntity", field="entity_id", through="TestManyToMany")]
pub struct MyEntity2 {
    #[primary_key]
    id: i32,

    name: String,

    #[indexed]
    integer: i32,
    integer64: i64,

    float: f32,
    double: f64,

    boolean: bool,

    datetime: Option<DateTime<Utc>>,
}
```

The above will produce helper methods to read and write the entities from db.

```rust
use oxidizer::db::DB;

...


let uri = "postgres://postgres:alkje2lkaj2e@db/postgres";
let max_open = 50; // mobc
let ca_file: Option<&str> = None;
let db = DB::connect(&uri, max_open, ca_file).await.unwrap();

db.migrate_tables(&[MyEntity::create_migration().unwrap()]).await.unwrap();

let mut entity = MyEntity { ... };
let creating = entity.save(&db).await.unwrap();
assert_eq!(creating, true);
```