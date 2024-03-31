use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Version};

use crate::{header::HeaderPair, Response};

use super::{Body, Parts};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ResponseBuilder {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status: StatusCode::default(),
            version: Version::HTTP_11,
            headers: HeaderMap::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn new_with_preset(
        status: StatusCode,
        http_version: Version,
        headers: HeaderMap,
    ) -> Self {
        Self {
            status,
            version: http_version,
            headers,
        }
    }

    pub fn body(self, body: Body) -> crate::Result<Response> {
        Ok(Response::from_parts(
            Parts::new(self.status, self.version, self.headers),
            body,
        ))
    }

    pub async fn body_path(self, path: impl AsRef<Path>) -> crate::Result<Response> {
        Response::from_path(path, Parts::new(self.status, self.version, self.headers)).await
    }
    pub fn body_file(self, file: crate::_fs::File) -> crate::Result<Response> {
        Ok(Response::from_file(
            file,
            Parts::new(self.status, self.version, self.headers),
        ))
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
