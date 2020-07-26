
pub enum Error {
    IOError(std::io::Error),
    ParsingError(syn::Error),
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error::ParsingError(err)
    }
}