
use barrel::{backend::Pg, Migration as RawMigration};

/// Migration abstract layer
pub struct Migration {
    pub name: String,

    pub raw: RawMigration,
}

impl Migration {
    /// Creates a new migration
    pub fn new(name: &str) -> Self {
        Migration{
            name: name.to_string(),

            raw: RawMigration::new(),
        }
    }

    /// Builds the raw query from the migration
    pub fn make(&self) -> String {
        self.raw.make::<Pg>()
    }
}


/// Creates a new migration module
#[macro_export]
macro_rules! create_migration_module {
    ($entity:ident) => {

        pub fn migration() -> String {
            let m = <$entity>::create_migration().expect(concat!("Could not create migration for ", stringify!($entity)));
            m.make()
        }

    };
}