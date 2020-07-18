use dboom_entity_macro::Entity;
use dboom::entity::Entity;
use crate as dboom;

use chrono::{DateTime, Utc};

#[derive(Entity)]
#[derive(Default)]
struct TestEntity {
    #[primary_key]
    id: i32,
    name: String,

    #[indexed]
    integer: i32,
    integer64: i64,

    float: f32,
    double: f64,

    boolean: bool,

    datetime: Option<chrono::DateTime<Utc>>,
}

#[derive(Entity)]
struct TestRelation {
    #[primary_key]
    id: i32,
    device_id: String,

    entity_id: i32,
}

#[derive(Entity)]
struct TestOnlyPK {
    #[primary_key]
    id: i32
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
    let db = super::db::test_utils::create_test_db("test_entity_macro_save", false).await;

    let m = TestEntity::create_migration().await.unwrap();
    let sql = m.make::<dboom::barrel::backend::Pg>();
    db.execute(&sql, &[]).await.unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, false);
}


#[tokio::test]
async fn test_entity_macro_find() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_find", false).await;

    let m = TestEntity::create_migration().await.unwrap();
    let sql = m.make::<dboom::barrel::backend::Pg>();
    db.execute(&sql, &[]).await.unwrap();

    let mut obj = TestEntity::default();
    obj.name = "test".to_string();
    let creating = obj.save(&db).await.unwrap();
    assert_eq!(creating, true);

    let result = TestEntity::find(&db, "id = $1", &[&obj.id]).await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_entity_macro_first() {
    let db = super::db::test_utils::create_test_db("test_entity_macro_first", false).await;

    let m = TestEntity::create_migration().await.unwrap();
    let sql = m.make::<dboom::barrel::backend::Pg>();
    db.execute(&sql, &[]).await.unwrap();

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
    let db = super::db::test_utils::create_test_db("test_entity_macro_delete", false).await;

    let m = TestEntity::create_migration().await.unwrap();
    let sql = m.make::<dboom::barrel::backend::Pg>();
    db.execute(&sql, &[]).await.unwrap();

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