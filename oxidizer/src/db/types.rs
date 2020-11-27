use quaint::prelude::*;
use std::convert::TryFrom;

use crate::error::*;

/// blanket
pub trait ToDBType<'a> {
    fn to_db_type(self) -> Value<'a>;
}

impl<'a, T> ToDBType<'a> for T
where
    T: Into<Value<'a>>,
{
    fn to_db_type(self) -> Value<'a> {
        self.into()
    }
}

pub trait ToDBTypeStringRef<'a> {
    fn to_db_type(self) -> Value<'a>;
}

impl<'a> ToDBTypeStringRef<'a> for &'a String {
    fn to_db_type(self) -> Value<'a> {
        self.as_str().into()
    }
}

pub trait DBTypeExtract<'a, T>
where
    T: TryFrom<Value<'a>>,
{
    fn extract(self) -> DBResult<T>;
}

impl<'a, T> DBTypeExtract<'a, T> for Option<&Value<'a>>
where
    T: TryFrom<Value<'a>>,
{
    fn extract(self) -> DBResult<T> {
        let value = self.ok_or(Error::Other("Could not extract value from row".to_string()))?;

        T::try_from(value.to_owned())
            .map_err(|_| Error::Other("could not extract value from row".to_string()))
    }
}
