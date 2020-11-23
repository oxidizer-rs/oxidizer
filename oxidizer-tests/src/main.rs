use oxidizer::*;
use sqlx::prelude::*;

//mod tmp;

#[derive(Entity, Default)]
pub struct TestCustomPrimaryKey {
    #[primary_key()]
    name: String,

    email: String,
}

#[derive(Entity, Default)]
pub struct TestCustomPrimaryKey2 {
    #[primary_key(increments)]
    id: i32,

    email: String,
}

fn main() {}

mod test {
    use super::*;

    // #[tokio::test]
    // async fn test_abc() {
    //     let mut abc = ABC::default();
    // }|
}
