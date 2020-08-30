use chrono;

#[tokio::test]
async fn test_db_raw_query() {
    let db = super::test_utils::create_test_db("test_db_raw_query").await;

    let query = "
    CREATE TABLE films (
        code        char(5) CONSTRAINT firstkey PRIMARY KEY,
        title       varchar(40) NOT NULL,
        date_prod   date,
        kind        varchar(10),
        nn  integer
    );
    ";
    db.execute(query, &[]).await.unwrap();

    let query = "
    insert into films (code, title, date_prod, kind, nn) values ($1, $2, $3, $4, $5)
    ";
    let rows_changed = db
        .execute(
            query,
            &[&"abcde", &"film title", &chrono::NaiveDate::from_ymd(2020, 8, 30), &"action", &(2 as i32)]
        )
        .await
        .unwrap();
    assert_eq!(1, rows_changed);

    let query = "select * from films where nn = $1";
    let row = db.query(query, &[&(2 as i32)]).await.unwrap();
    assert_eq!(1, row.len());
    assert_eq!("abcde", row[0].get::<&str, &str>("code"));
}