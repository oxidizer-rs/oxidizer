use oxidizer::*;

// mod test_2;

#[derive(PartialEq, Debug)]
pub enum MyEnum {
    Item1,
    Item2,
}

impl Default for MyEnum {
    fn default() -> Self {
        MyEnum::Item1
    }
}

impl std::convert::From<&MyEnum> for i32 {
    fn from(v: &MyEnum) -> Self {
        match v {
            MyEnum::Item1 => 0,
            MyEnum::Item2 => 1,
        }
    }
}

impl std::convert::From<i32> for MyEnum {
    fn from(v: i32) -> Self {
        match v {
            0 => MyEnum::Item1,
            1 => MyEnum::Item2,
            _ => unimplemented!(),
        }
    }
}

#[derive(Entity, Default)]
pub struct TestCustomType {
    #[primary_key]
    id: i32,

    #[custom_type(ty = "i32")]
    my_enum: MyEnum,
}

fn main() {}

mod test {
    use super::*;

    // #[tokio::test]
    // async fn test_abc() {
    //     let mut abc = ABC::default();
    // }|
}
