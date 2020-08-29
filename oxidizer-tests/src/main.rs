use oxidizer::*;

#[derive(Entity, Default)]
pub struct ABC {
    #[primary_key]
    pub id: i32,
    pub name: String,
    pub addr1: String,
    pub addr2: String,
}

fn main() {}

mod test {
    use super::*;

    #[tokio::test]
    async fn test_abc() {
        let mut abc = ABC::default();
    }
}
