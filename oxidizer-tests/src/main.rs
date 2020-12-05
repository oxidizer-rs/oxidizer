use oxidizer::*;

mod tmp;

#[derive(Entity, Default)]
pub struct TestCustomPrimaryKey {
    #[primary_key()]
    name: i32,

    email: String,
}

//#[derive(Entity, Default)]
//pub struct TestCustomPrimaryKey2 {
//#[primary_key(increments)]
//id: i32,

//email: String,
//}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    // #[tokio::test]
    // async fn test_abc() {
    //     let mut abc = ABC::default();
    // }|

    ///
    /// Integration test for postgres TLS support (manual invocation only)
    ///
    /// Requires:
    ///  - run of ./generate-pki.sh in ./.devcontainer/dev-pki
    ///  - start of ./.devcontainer/docker-compose-tls.yml db
    ///
    //#[tokio::test]
    async fn test_tls() {
        let db = DB::connect(
            "postgres://postgres:alkje2lkaj2e@localhost:5432/postgres",
            50,
            Some("../.devcontainer/dev-pki/ca.cert"),
        )
        .await
        .unwrap();

        //db.migrate_tables(&[TestCustomPrimaryKey::create_migration().unwrap()])
        //.await
        //.unwrap();

        let mut entity = TestCustomPrimaryKey::default();
        let creating = entity.save(&db).await.unwrap();
        assert_eq!(creating, true);
    }
}
