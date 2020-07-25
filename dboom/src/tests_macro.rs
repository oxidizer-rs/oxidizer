use dboom_entity_macro::Entity;
use dboom::entity::Entity;
use crate as dboom;

use chrono::{DateTime, Utc};

#[derive(Entity)]
#[derive(Default)]
pub struct TestEntity {
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

#[derive(Entity)]
struct TestRelation {
    #[primary_key]
    id: i32,
    device_id: String,

    #[relation(model="TestEntity", key="id")]
    entity_id: i32,
}

#[derive(Entity)]
struct TestOnlyPK {
    #[primary_key]
    id: i32
}

#[derive(Default, Entity)]
struct TestNullable {
    #[primary_key]
    id: i32,

    name: Option<String>,
}

#[derive(Default, Entity)]
struct TestNullableRelation {
    #[primary_key]
    id: i32,

    #[relation(model="TestEntity", key="id")]
    entity_id: Option<i32>,
}

#[derive(Default, Entity)]
#[entity(table_name="custom")]
struct TestCustomTableName {
    #[primary_key]
    id: i32,
}

#[derive(Default, Entity)]
#[entity(table_name="custom2")]
#[index(name="myindex", columns="name, date", unique)]
#[index(name="myindex2", columns="email", unique)]
struct TestCustomIndexes {
    #[primary_key]
    id: i32,

    name: String,
    date: String,
    email: String,
}

#[tokio::test]
async fn test_entity_macro_clean() {
    let _obj = TestEntity{
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

#[tokio::test]
async fn test_entity_macro_save() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_save").await;

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);
}


#[tokio::test]
async fn test_entity_macro_find() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_find").await;

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

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

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let result = TestEntity::first(&db, "id = $1", &[&obj.id]).await.unwrap().unwrap();
    assert_eq!(result.id, obj.id);

    let id: i32 = 2;
    let result = TestEntity::first(&db, "id = $1", &[&id]).await.unwrap();
    assert!(result.is_none())
}

#[tokio::test]
async fn test_entity_macro_delete() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_delete").await;

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

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

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let m = TestRelation::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut entity = TestEntity::default();
    entity.name = "test".to_string();
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut entity2 = TestEntity::default();
    entity2.name = "test 2".to_string();
    let creating = entity2.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut obj = TestRelation{
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

    let m = TestNullable::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut obj = TestNullable::default();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    assert_eq!(None, obj.name);

    let loaded = TestNullable::first(&db, "id = $1", &[&obj.id]).await.unwrap().unwrap();
    assert_eq!(None, loaded.name);

    obj.name = Some("test".to_string());
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);

    let loaded = TestNullable::first(&db, "id = $1", &[&obj.id]).await.unwrap().unwrap();
    assert_eq!(Some("test".to_string()), loaded.name);
}

#[tokio::test]
async fn test_relation_nullable() {
    let db = super::db::test_utils::create_test_db("test_relation_nullable").await;

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let m = TestNullableRelation::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut entity = TestEntity::default();
    entity.name = "test".to_string();
    let creating = entity.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let mut obj = TestNullableRelation{
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

    let m = TestCustomIndexes::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();

    let mut obj = TestCustomIndexes{
        id: 0,
        name: "test".to_string(),
        date: "07/19/2020".to_string(),
        email: "me@example.com".to_string(),

    };
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(true, creating);

    let mut obj2 = TestCustomIndexes{
        id: 0,
        name: "test".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),

    };
    assert!(obj2.save(&db).await.is_err());

    let mut obj2 = TestCustomIndexes{
        id: 0,
        name: "test2".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),

    };
    assert!(obj2.save(&db).await.is_ok());

    let mut obj2 = TestCustomIndexes{
        id: 0,
        name: "test3".to_string(),
        date: "07/19/2020".to_string(),
        email: "me2@example.com".to_string(),

    };
    assert!(obj2.save(&db).await.is_err());

    let mut obj2 = TestCustomIndexes{
        id: 0,
        name: "test3".to_string(),
        date: "07/19/2020".to_string(),
        email: "me3@example.com".to_string(),

    };
    assert!(obj2.save(&db).await.is_ok());
}

#[tokio::test]
async fn test_safe_migration() {
    let db = super::db::test_utils::create_test_db("test_safe_migration").await;

    let m = TestEntity::create_migration().await.unwrap();
    db.migrate_table(&m).await.unwrap();
    db.migrate_table(&m).await.unwrap();
}