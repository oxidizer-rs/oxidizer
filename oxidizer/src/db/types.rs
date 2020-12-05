use quaint::prelude::*;
use std::convert::TryFrom;

use crate::error::*;

/// blanket
pub trait ToCompatibleType<'a> {
    fn to_compatible_type(self) -> Value<'a>;
}

impl<'a, T> ToCompatibleType<'a> for T
where
    T: Into<Value<'a>>,
{
    fn to_compatible_type(self) -> Value<'a> {
        self.into()
    }
}

pub trait ToCompatibleTypeStringRef<'a> {
    fn to_compatible_type(self) -> Value<'a>;
}

impl<'a> ToCompatibleTypeStringRef<'a> for &'a String {
    fn to_compatible_type(self) -> Value<'a> {
        self.as_str().into()
    }
}

pub trait CompatibleTypeExtract<'a, T>
where
    T: TryFrom<Value<'a>>,
{
    fn extract(self) -> DBResult<T>;
}

impl<'a, T> CompatibleTypeExtract<'a, T> for Option<&Value<'a>>
where
    T: TryFrom<Value<'a>>,
{
    fn extract(self) -> DBResult<T> {
        let value = self.ok_or(Error::Other("Could not extract value from row".to_string()))?;

        T::try_from(value.to_owned())
            .map_err(|_| Error::Other("could not extract value from row".to_string()))
    }
}
