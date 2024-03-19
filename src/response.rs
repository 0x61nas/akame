pub mod builder;

use std::{
    io::Write as StdWrite,
    ops::{Deref, DerefMut},
};

pub use http::response::Parts;
use http::Version;
use tokio::io::AsyncWriteExt;

use crate::{Result, CRLF};

pub use self::builder::ResponseBuilder;

macro_rules! for_now {
    ($msg: expr; $value: expr) => {{
        eprintln!("WARINIG: for now, {}", $msg);
        $value
    }};
    ($value: expr) => {
        for_now!("its conistant, but remember to change it."; $value)
    };
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Response {
    inner: http::Response<Vec<u8>>,
}

type Body = Vec<u8>;

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
    #[inline(always)]
    pub fn into_parts(self) -> (Parts, Body) {
        self.inner.into_parts()
    }

    pub fn from_parts(head: Parts, body: Body) -> Self {
        Self {
            inner: http::Response::from_parts(head, body),
        }
    }

    fn from_inner(inner: http::Response<Vec<u8>>) -> Self {
        Self { inner }
    }

    pub(crate) async fn write<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::with_capacity(for_now!(524));
        // NOTE: maybe use `write_vectored`?
        // The header
        self.write_status_line(&mut buf)?;
        self.write_headers(&mut buf)?;

        // The body
        StdWrite::write_all(&mut buf, CRLF.as_bytes())?;
        StdWrite::write_all(&mut buf, self.inner.body())?;
        // StdWrite::write_all(&mutbuf, b'\0')?;

        // write
        writer.write_all(&buf).await.map_err(crate::Error::IOError)
    }

    #[inline]
    fn write_status_line(&self, writer: &mut impl StdWrite) -> Result<()> {
        write!(
            writer,
            "HTTP/{} {} {}{CRLF}",
            match self.version() {
                Version::HTTP_2 => "2.0",
                Version::HTTP_11 => "1.1",
                Version::HTTP_3 => "3.0",
                Version::HTTP_09 => "0.9",
                Version::HTTP_10 => "1.0",
                _ => "1.1",
            },
            self.status().as_str(),
            for_now!("OK")
        )
        .map_err(crate::Error::IOError)
    }

    #[inline]
    fn write_headers(&self, writer: &mut impl StdWrite) -> Result<()> {
        for (header, val) in self.inner.headers().iter() {
            StdWrite::write_fmt(
                writer,
                format_args!(
                    "{}: {}{CRLF}",
                    header.as_str(),
                    val.to_str()
                        .map_err(|_| crate::Error::InvalidHeader(header.as_str().to_string()))?
                ),
            )?;
        }
        Ok(())
    }
}

impl Deref for Response {
    type Target = http::Response<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// impl From<http::Response<Vec<u8>>> for Response {
//     #[inline(always)]
//     fn from(inner: http::Response<Vec<u8>>) -> Self {
//         Self { inner }
//     }
// }

impl<T> From<T> for Response
where
    T: Deref<Target = str>,
{
    #[inline(always)]
    fn from(body: T) -> Self {
        Self::builder()
            .body(body.as_bytes().to_vec())
            .expect("Unreachable")
    }
}
