use super::*;

mod oxidizer {
    pub use crate::*;
}

#[derive(Default, Entity)]
struct TestTransaction {
    #[primary_key(increments)]
    id: i32,

    name: String,
}

#[tokio::test]
async fn test_transaction_save() {
    let db = super::db::test_utils::create_test_db("test_transaction_save").await;

    db.migrate_tables(&[TestTransaction::create_migration().unwrap()])
        .await
        .unwrap();

    db.with_transaction(|t| async {
        let mut obj = TestTransaction::default();
        obj.name = "test".to_string();
        let creating = obj.save(t).await.unwrap();
        assert_eq!(creating, true);

        let creating = obj.save(t).await.unwrap();
        assert_eq!(creating, false);

        Ok(())
    })
    .await
    .unwrap();
}
