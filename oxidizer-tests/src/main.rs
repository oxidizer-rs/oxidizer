use oxidizer::*;

// mod tmp;

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

pub enum ConvertError {
    Error,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error trying to convert")
    }
}

impl TryFrom<&MyEnum> for i32 {
    type Error = ConvertError;

    fn try_from(v: &MyEnum) -> Result<Self, Self::Error> {
        match v {
            MyEnum::Item1 => Ok(0),
            MyEnum::Item2 => Ok(1),
        }
    }
}

impl TryFrom<i32> for MyEnum {
    type Error = ConvertError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(MyEnum::Item1),
            1 => Ok(MyEnum::Item2),
            _ => Err(ConvertError::Error),
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
