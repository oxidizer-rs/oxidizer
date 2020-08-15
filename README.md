# Oxidizer

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

## Overview

A Rust ORM based on [tokio-postgres](https://crates.io/crates/tokio-postgres) and [refinery](https://crates.io/crates/refinery) that helps you reduce the boilerplate of writing entities, tables & migrations when using tokio-postgres and refinery.

- Asynchronous from the ground up. All the database operations are
  efficiently handled by tokio at runtime.
- Oxidizer macros generate code to access relations between entities with ease. Forward and reverse relations are supported.

> Note that, while functional and working, this is in early stages. Use with caution.

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

## Contributing

There are a couple of ways in which you can contribute to Oxidizer, for example:

- [Submit bugs and feature requests](https://github.com/oxidizer-rs/oxidizer/issues), and help us verify as they are checked in
- Review the [documentation](https://oxidizer.rs/docs) and make pull requests for anything from typos to new content suggestion

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Oxidizer by you, shall be licensed as MIT, without any additional
terms or conditions.

## Feedback

- Ask a question on [Stack Overflow](https://stackoverflow.com/questions/tagged/oxidizer-rs)
- [Report an issue](https://github.com/oxidizer-rs/oxidizer/issues)
- Up vote [popular feature requests](https://github.com/oxidizer-rs/oxidizer/issues?q=is%3Aopen+is%3Aissue+label%3Afeature-request+sort%3Areactions-%2B1-desc)

## License

This project is licensed under the [MIT license].

[mit license]: https://github.com/oxidizer-rs/oxidizer/blob/master/LICENSE
