use dboom_entity_macro::Entity;
use crate as dboom;

#[derive(Entity)]
#[derive(Default)]
struct TestEntity {
    #[primary_key]
    id: i32,
    name: String,
}

#[tokio::test]
async fn test_entity_macro_clean() {
    let _obj = TestEntity{
        id: 0,
        name: "test".to_string(),
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