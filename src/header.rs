use std::{ops::Deref, str::FromStr};

pub use http::header::*;
use mime::Mime;

#[derive(thiserror::Error, Debug)]
#[error("Cant parse the provided header")]
pub struct HeaderParseError;

pub struct HeaderPair {
    pub name: HeaderName,
    pub value: HeaderValue,
}

pub struct ContentType(pub Mime);

impl std::ops::Deref for ContentType {
    type Target = Mime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ContentType> for HeaderPair {
    fn from(value: ContentType) -> Self {
        Self {
            name: CONTENT_TYPE,
            value: HeaderValue::from_str(value.deref().essence_str()).expect("Unreatchable"),
        }
    }
}

impl TryFrom<&str> for HeaderPair {
    type Error = HeaderParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some((k, v)) = value.split_once(':') else {
            return Err(HeaderParseError);
        };
        Ok(Self {
            name: HeaderName::from_str(k).map_err(|_| HeaderParseError)?,
            value: v.try_into().map_err(|_| HeaderParseError)?,
        })
    }
}
