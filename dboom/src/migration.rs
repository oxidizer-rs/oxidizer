
use barrel::{backend::Pg, Migration as RawMigration};

pub struct Migration {
    pub name: String,

    pub raw: RawMigration,
}

impl Migration {
    pub fn new(name: &str) -> Self {
        Migration{
            name: name.to_string(),

            raw: RawMigration::new(),
        }
    }

    pub fn make(&self) -> String {
        self.raw.make::<Pg>()
    }
}


#[macro_export]
macro_rules! create_migration {
    ($entity:ident) => {

        pub fn migration() -> String {
            let m = <$entity>::create_migration().expect(concat!("Could not create migration for ", stringify!($entity)));
            m.make()
        }

    };
}