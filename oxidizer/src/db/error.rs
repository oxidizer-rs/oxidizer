#[derive(Debug)]
pub enum Error {
    DoesNotExist,
    ReferencedModelIsNotInDB,
    Other(String),
}

pub type DBResult<T> = std::result::Result<T, Error>;

impl<R> std::convert::From<R> for Error
where
    R: std::fmt::Display,
{
    fn from(v: R) -> Self {
        Error::Other(v.to_string())
    }
}
