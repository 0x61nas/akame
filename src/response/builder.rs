use std::ops::{Deref, DerefMut};

use http::response::Builder as HttpBuilder;

use crate::Response;

use super::Body;

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    inner: HttpBuilder,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            inner: HttpBuilder::new(),
        }
    }

    pub fn body(self, body: Body) -> crate::Result<Response> {
        Ok(Response::from_inner(self.inner.body(body).map_err(
            |e| crate::Error::InvalidResponse(e.to_string()),
        )?))
    }
}

impl Deref for ResponseBuilder {
    type Target = HttpBuilder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ResponseBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
