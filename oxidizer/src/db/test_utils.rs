use super::*;

pub async fn create_test_db(name: &str) -> DB {
    let uri = "postgres://postgres:alkje2lkaj2e@db/postgres";
    let db = DB::connect(&uri, 50, None).await.unwrap();

    let query_str = format!("DROP DATABASE IF EXISTS db_test_{}", name.to_lowercase());
    db.execute(&query_str, args![]).await.unwrap();
    let query_str = format!("CREATE DATABASE db_test_{}", name.to_lowercase());
    db.execute(&query_str, args![]).await.unwrap();

    drop(db);

    let uri = format!("postgres://postgres:alkje2lkaj2e@db/db_test_{}", name);
    let db = DB::connect(&uri, 50, None).await.unwrap();

    db
}
