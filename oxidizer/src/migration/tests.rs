use crate::test_utils;

use crate::db::types::*;
use crate::migration::*;
use crate::GenericClient;

#[tokio::test]
async fn test_create_runner_from_directory() {
    let db = test_utils::create_test_db("test_create_runner_from_directory").await;

    let mut cwd = std::env::current_dir().unwrap();
    cwd.push("..");
    cwd.push("migration_tests");

    let runner = Runner::from_directory(cwd).await.unwrap();

    assert_eq!(runner.migrations.len(), 2);
}

#[tokio::test]
async fn test_execute_migrations() {
    let db = test_utils::create_test_db("test_execute_migrations").await;

    let mut cwd = std::env::current_dir().unwrap();
    cwd.push("..");
    cwd.push("migration_tests");

    let runner = Runner::from_directory(cwd).await.unwrap();

    assert_eq!(runner.migrations.len(), 2);

    let before_timestamp = chrono::Utc::now();
    assert_eq!(runner.execute(&db).await.unwrap(), 2);

    for migration in runner.migrations.iter() {
        let results = db
            .query(
                "select * from oxidizer_migrations where filename = $1",
                &[migration.filename.to_str().unwrap().to_compatible_type()],
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        let db_hash: String = results.get(0).unwrap().get("hash").extract().unwrap();
        assert_eq!(db_hash, migration.hash);

        let executed_at: chrono::DateTime<chrono::Utc> = results
            .get(0)
            .unwrap()
            .get("executed_at")
            .extract()
            .unwrap();

        assert!(before_timestamp.lt(&executed_at));
        assert!(chrono::Utc::now().gt(&executed_at));
    }
}
