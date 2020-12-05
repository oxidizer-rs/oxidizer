#[derive(Debug)]
pub enum MigrationError {
    FileDoesNotExist(String),
}
