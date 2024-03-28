use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Version};

use crate::{header::HeaderPair, Response};

use super::{Body, Parts};

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    status: StatusCode,
    version: Version,
    headers: HeaderMap,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status: StatusCode::default(),
            version: Version::HTTP_2,
            headers: HeaderMap::default(),
        }
    }

    pub fn body(self, body: Body) -> crate::Result<Response> {
        Ok(Response::from_parts(
            Parts::new(self.status, self.version, self.headers),
            body,
        ))
    }

    pub async fn file(self, path: impl AsRef<Path>) -> crate::Result<Response> {
        Response::from_file(path, Parts::new(self.status, self.version, self.headers)).await
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn header(mut self, name: HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.headers.insert(name, value.into());
        self
    }

    pub fn header_pair(mut self, pair: impl Into<HeaderPair>) -> Self {
        let pair = pair.into();
        self.headers.insert(pair.name, pair.value);
        self
    }
}

// impl Deref for ResponseBuilder {
//     type Target = HttpBuilder;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl DerefMut for ResponseBuilder {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

#[macro_export]
macro_rules! not_found {
    () => {
        not_found!(::std::vec![])
    };
    ($body: expr) => {
        $crate::Response::builder()
            .status($crate::StatusCode::NOT_FOUND)
            .body($body)
    };
}
