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
    let obj = TestEntity{
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