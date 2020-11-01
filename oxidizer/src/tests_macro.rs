use super::*;

mod oxidizer {
    pub use crate::*;
}

use chrono::{DateTime, Utc};

#[derive(Entity, Default)]
struct TestPKNoIncrements {
    #[primary_key()]
    id: i32,
}

#[derive(Entity, Default)]
pub struct TestEntity {
    #[primary_key(increments)]
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

#[derive(Entity)]
struct TestRelation {
    #[primary_key(increments)]
    id: i32,
    device_id: String,

    #[relation(model = "TestEntity", key = "id")]
    entity_id: i32,
}

#[derive(Entity, Default)]
struct TestOnlyPK {
    #[primary_key(increments)]
    id: i32,
}

#[derive(Default, Entity)]
struct TestNullable {
    #[primary_key(increments)]
    id: i32,

    name: Option<String>,
}

#[derive(Default, Entity)]
struct TestNullableRelation {
    #[primary_key(increments)]
    id: i32,

    #[relation(model = "TestEntity", key = "id")]
    entity_id: Option<i32>,
}

#[derive(Default, Entity)]
#[entity(table_name = "custom")]
struct TestCustomTableName {
    #[primary_key(increments)]
    id: i32,
}

#[derive(Default, Entity)]
#[entity(table_name = "custom2")]
#[index(name = "myindex", columns = "name, date", unique)]
#[index(name = "myindex2", columns = "email", unique)]
struct TestCustomIndexes {
    #[primary_key(increments)]
    id: i32,

    name: String,
    date: String,
    email: String,
}

#[derive(Default, Entity)]
pub struct TestReverseRelation {
    #[primary_key(increments)]
    id: i32,

    #[relation(model = "TestReverseRelationTarget", key = "id")]
    entity_id: i32,
}

#[derive(Default, Entity)]
#[has_many(model = "TestReverseRelation", field = "entity_id")]
#[has_many(model = "TestEntity", field = "entity_id", through = "TestManyToMany")]
pub struct TestReverseRelationTarget {
    #[primary_key(increments)]
    id: i32,
}

#[derive(Default, Entity)]
pub struct TestManyToMany {
    #[primary_key(increments)]
    id: i32,

    #[relation(model = "TestReverseRelationTarget", key = "id")]
    target_id: i32,

    #[relation(model = "TestEntity", key = "id")]
    entity_id: i32,
}

#[derive(Default)]
pub struct TestIgnoredType {
    data: i32,
}

#[derive(Entity, Default)]
pub struct TestIgnoreField {
    #[primary_key(increments)]
    id: i32,
    name: String,

    #[field_ignore]
    ignored: TestIgnoredType,
}

#[derive(PartialEq, Debug)]
pub enum MyEnum {
    Item1,
    Item2,
}

impl Default for MyEnum {
    fn default() -> Self {
        MyEnum::Item1
    }
}

pub enum ConvertError {
    Error,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error trying to convert")
    }
}

impl std::convert::TryFrom<&MyEnum> for i32 {
    type Error = ConvertError;

    fn try_from(v: &MyEnum) -> Result<Self, Self::Error> {
        match v {
            MyEnum::Item1 => Ok(0),
            MyEnum::Item2 => Ok(1),
        }
    }
}

impl std::convert::TryFrom<i32> for MyEnum {
    type Error = ConvertError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(MyEnum::Item1),
            1 => Ok(MyEnum::Item2),
            _ => Err(ConvertError::Error),
        }
    }
}

#[derive(Entity, Default)]
pub struct TestCustomType {
    #[primary_key(increments)]
    id: i32,

    #[custom_type(ty = "i32")]
    my_enum: MyEnum,
}

#[derive(Entity, Default)]
pub struct TestCustomPrimaryKey {
    #[primary_key(increments = "false")]
    name: String,

    email: String,
}

#[tokio::test]
async fn test_entity_macro_clean() {
    let _obj = TestEntity {
        id: 0,
        name: "test".to_string(),
        integer: 0,
        integer64: 0,
        float: 0.0,
        double: 0.0,
        boolean: false,

        datetime: None,
    };
}

mod migration_modules {
    use super::*;
    use crate::create_migration_module;

    create_migration_module!(TestEntity);
}

#[tokio::test]
async fn test_entity_macro_save() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_save").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);
}

#[tokio::test]
async fn test_entity_macro_save_update() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_save_update").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestEntity::default();
    obj.integer = 42;
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);
    assert_eq!(obj.id, 1);

    let mut obj2 = TestEntity::first(&db, "integer = $1", &[&obj.integer])
        .await
        .unwrap()
        .unwrap();

    obj2.integer = 43;
    let creating = obj2.save(&db).await.unwrap();
    assert_eq!(creating, false);
    assert_eq!(obj2.id, 1);
}

#[tokio::test]
async fn test_entity_macro_find() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_find").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let result = TestEntity::find(&db, "id = $1", &[&obj.id]).await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_entity_macro_first() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_first").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let result = TestEntity::first(&db, "id = $1", &[&obj.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.id, obj.id);

    let id: i32 = 2;
    let result = TestEntity::first(&db, "id = $1", &[&id]).await.unwrap();
    assert!(result.is_none())
}

#[tokio::test]
async fn test_entity_macro_delete() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_delete").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    assert!(obj.delete(&db).await.unwrap());
    assert_eq!(obj.id, 0);
    obj.id = 1;

    assert_eq!(false, obj.delete(&db).await.unwrap());
    obj.id = 1;

    obj.id = 0;
    assert_eq!(false, obj.delete(&db).await.unwrap());

    let result = TestEntity::first(&db, "id = $1", &[&obj.id]).await.unwrap();
    assert!(result.is_none())
}

#[tokio::test]
async fn test_relation() {
    let db = super::db::test_utils::create_test_db("test_relation").await;

    db.migrate_tables(&[
        TestEntity::create_migration().unwrap(),
        TestRelation::create_migration().unwrap(),
    ])
    .await
    .unwrap();

    let mut entity = TestEntity::default();
    entity.name = "test".to_string();
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut entity2 = TestEntity::default();
    entity2.name = "test 2".to_string();
    let creating = entity2.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut obj = TestRelation {
        id: 0,
        device_id: "abc12".to_string(),
        entity_id: entity.id,
    };
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let loaded = obj.get_test_entity(&db).await.unwrap();
    assert_eq!(entity.id, loaded.id);

    obj.set_test_entity(&db, &entity2).await.unwrap();

    let loaded = obj.get_test_entity(&db).await.unwrap();
    assert_eq!(entity2.id, loaded.id);
}

#[tokio::test]
async fn test_nullable() {
    let db = super::db::test_utils::create_test_db("test_nullable").await;

    db.migrate_tables(&[TestNullable::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestNullable::default();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    assert_eq!(None, obj.name);

    let loaded = TestNullable::first(&db, "id = $1", &[&obj.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(None, loaded.name);

    obj.name = Some("test".to_string());
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let loaded = TestNullable::first(&db, "id = $1", &[&obj.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(Some("test".to_string()), loaded.name);
}

#[tokio::test]
async fn test_relation_nullable() {
    let db = super::db::test_utils::create_test_db("test_relation_nullable").await;

    db.migrate_tables(&[
        TestEntity::create_migration().unwrap(),
        TestNullableRelation::create_migration().unwrap(),
    ])
    .await
    .unwrap();

    let mut entity = TestEntity::default();
    entity.name = "test".to_string();
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut obj = TestNullableRelation {
        id: 0,
        entity_id: None,
    };
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    assert!(obj.get_test_entity(&db).await.is_err());

    obj.set_test_entity(&db, &entity).await.unwrap();

    let loaded = obj.get_test_entity(&db).await.unwrap();
    assert_eq!(entity.id, loaded.id);
}

#[tokio::test]
async fn test_custom_table_name() {
    assert_eq!("custom", TestCustomTableName::get_table_name());
}

#[tokio::test]
async fn test_indexes() {
    let db = super::db::test_utils::create_test_db("test_indexes").await;

    db.migrate_tables(&[TestCustomIndexes::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestCustomIndexes {
        id: 0,
        name: "test".to_string(),
        date: "07/19/2020".to_string(),
        email: "me@example.com".to_string(),
    };
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(true, creating);

    let mut obj2 = TestCustomIndexes {
        id: 0,
        name: "test".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),
    };
    assert!(obj2.save(&db).await.is_err());

    let mut obj2 = TestCustomIndexes {
        id: 0,
        name: "test2".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),
    };
    assert!(obj2.save(&db).await.is_ok());

    let mut obj2 = TestCustomIndexes {
        id: 0,
        name: "test3".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),
    };
    assert!(obj2.save(&db).await.is_err());

    let mut obj2 = TestCustomIndexes {
        id: 0,
        name: "test3".to_string(),
        date: "07/19/2020".to_string(),
        email: "me3@example.com".to_string(),
    };
    assert!(obj2.save(&db).await.is_ok());
}

#[tokio::test]
async fn test_safe_migrations() {
    let db = super::db::test_utils::create_test_db("test_safe_migrations").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();
    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    #[derive(Entity)]
    #[entity(table_name = "test_entity")]
    struct TestEntityChanged {
        #[primary_key(increments)]
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

    // Hash should match
    db.migrate_tables(&[TestEntityChanged::create_migration().unwrap()])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_migrations_changed() {
    let db = super::db::test_utils::create_test_db("test_migrations_changed").await;

    db.migrate_tables(&[TestEntity::create_migration().unwrap()])
        .await
        .unwrap();

    #[derive(Entity)]
    #[entity(table_name = "test_entity")]
    struct TestEntityChanged {
        #[primary_key(increments)]
        id: i32,
        name: String,

        #[indexed]
        integer: i32,
        integer64: i64,

        float: f32,
        double: f64,

        boolean: bool,

        datetime: Option<DateTime<Utc>>,

        new_field: bool,
    }

    db.migrate_tables(&[TestEntityChanged::create_migration().unwrap()])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_migrations_module() {
    let migration = migration_modules::migration();
    assert_eq!(TestEntity::create_migration().unwrap().make(), migration);
}

#[tokio::test]
async fn test_migrations_module_fs() {
    let db = super::db::test_utils::create_test_db("test_migrations_module_fs").await;

    let runner = super::migrations::runner();
    let report = db.migrate(runner).await.unwrap();
    assert_eq!(1, report.applied_migrations().len());

    let mut entity = TestEntity::default();
    entity.name = "test".to_string();
    entity.save(&db).await.unwrap();
}

#[tokio::test]
async fn test_relation_has_many() {
    let db = super::db::test_utils::create_test_db("test_relation_has_many").await;

    db.migrate_tables(&[
        TestReverseRelationTarget::create_migration().unwrap(),
        TestReverseRelation::create_migration().unwrap(),
    ])
    .await
    .unwrap();

    let mut target = TestReverseRelationTarget::default();
    let creating = target.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut entity = TestReverseRelation::default();
    entity.entity_id = target.id;
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut entity2 = TestReverseRelation::default();
    entity2.entity_id = target.id;
    let creating = entity2.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let loaded = target.get_all_test_reverse_relation(&db).await.unwrap();
    assert_eq!(2, loaded.len());

    assert_eq!(entity.id, loaded[0].id);
    assert_eq!(entity2.id, loaded[1].id);
}

#[tokio::test]
async fn test_many_to_many() {
    let db = super::db::test_utils::create_test_db("test_many_to_many").await;

    db.migrate_tables(&[
        TestEntity::create_migration().unwrap(),
        TestReverseRelationTarget::create_migration().unwrap(),
        TestManyToMany::create_migration().unwrap(),
    ])
    .await
    .unwrap();

    let mut target = TestReverseRelationTarget::default();
    let creating = target.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut entity = TestEntity::default();
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut m2m = TestManyToMany::default();
    m2m.entity_id = entity.id;
    m2m.target_id = target.id;
    let creating = m2m.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let loaded_entity = target.get_all_test_entity(&db).await.unwrap();
    assert_eq!(1, loaded_entity.len());

    assert_eq!(entity.id, loaded_entity[0].entity_id);
}

#[tokio::test]
async fn test_entity_field_ignore() {
    let db = super::db::test_utils::create_test_db("test_entity_field_ignore").await;

    db.migrate_tables(&[TestIgnoreField::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestIgnoreField::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);
}

#[tokio::test]
async fn test_entity_custom_type() {
    let db = super::db::test_utils::create_test_db("test_entity_custom_type").await;

    db.migrate_tables(&[TestCustomType::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestCustomType::default();
    obj.my_enum = MyEnum::Item2;
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let result = TestCustomType::first(&db, "id = $1", &[&obj.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.my_enum, MyEnum::Item2);
}

#[tokio::test]
async fn test_entity_custom_type_error() {
    let db = super::db::test_utils::create_test_db("test_entity_custom_type_error").await;

    db.migrate_tables(&[TestCustomType::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestCustomType::default();
    obj.my_enum = MyEnum::Item2;
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let query = format!(
        "update {} set my_enum = $1",
        TestCustomType::get_table_name()
    );
    let value: i32 = 33;
    db.execute(&query, &[&value]).await.unwrap();

    let result = TestCustomType::first(&db, "id = $1", &[&obj.id]).await;
    assert_eq!(true, result.is_err());

    let query = format!(
        "update {} set my_enum = $1",
        TestCustomType::get_table_name()
    );
    let value: i32 = 0;
    db.execute(&query, &[&value]).await.unwrap();

    let result = TestCustomType::first(&db, "id = $1", &[&obj.id]).await;
    assert_eq!(true, result.is_ok());
}

#[tokio::test]
async fn test_only_pk() {
    let db = super::db::test_utils::create_test_db("test_only_pk").await;

    db.migrate_tables(&[TestOnlyPK::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestOnlyPK::default();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let result = TestOnlyPK::first(&db, "id = $1", &[&obj.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.id, obj.id);
}

#[tokio::test]
async fn test_custom_primary_key() {
    let db = super::db::test_utils::create_test_db("test_custom_primary_key").await;

    db.migrate_tables(&[TestCustomPrimaryKey::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestCustomPrimaryKey::default();
    obj.name = "hello".to_string();
    obj.email = "world".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let result = TestCustomPrimaryKey::first(&db, "name = $1", &[&obj.name])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.email, obj.email);
}

#[tokio::test]
async fn test_pk_no_increments() {
    let db = super::db::test_utils::create_test_db("test_pk_no_increments").await;

    db.migrate_tables(&[TestPKNoIncrements::create_migration().unwrap()])
        .await
        .unwrap();

    let mut obj = TestPKNoIncrements::default();
    obj.id = 1;
    obj.save(&db).await.unwrap();

    obj.id = 2;
    obj.save(&db).await.unwrap();

    let results = TestPKNoIncrements::find(&db, "true", &[]).await.unwrap();
    assert_eq!(results.len(), 2);
}
