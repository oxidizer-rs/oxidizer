
use barrel::Migration as RawMigration;

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
}