pub mod builder;

use std::{io::Write as StdWrite, mem, path::Path};

use http::{HeaderMap, HeaderValue, StatusCode, Version};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    header::{self, ContentType, HeaderPair},
    Result, CRLF,
};

pub use self::builder::ResponseBuilder;

const STATUS_LINE_SIZE: usize = 9 + 3 + 32 + 3;

#[macro_export]
macro_rules! for_now {
    ($msg: expr; $value: expr) => {{
        eprintln!("WARINIG: for now, {}", $msg);
        $value
    }};
    ($value: expr) => {
        for_now!("its conistant, but remember to change it."; $value)
    };
}

type Body = Vec<u8>;

#[derive(Clone)]
pub struct Parts {
    /// The response's status
    pub status: StatusCode,

    /// The response's version
    pub version: Version,

    /// The response's headers
    pub headers: HeaderMap<HeaderValue>,
    // /// The response's extensions
    // pub extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(status: StatusCode, version: Version, headers: HeaderMap) -> Self {
        Self {
            status,
            version,
            headers,
        }
    }
}

// #[derive(Clone)]
pub struct Response {
    // inner: http::Response<Vec<u8>>,
    head: Parts,
    pub(crate) body: Body,
    pub(crate) file: Option<File>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
    #[inline(always)]
    pub fn into_parts(self) -> (Parts, Body) {
        (self.head, self.body)
    }

    pub fn from_parts(head: Parts, body: Body) -> Self {
        Self {
            // inner: http::Response::from_parts(head, body),
            head,
            body,
            file: None,
        }
    }

    // fn from_inner(inner: http::Response<Vec<u8>>) -> Self {
    //     Self { inner, file: None }
    // }

    async fn from_path(path: impl AsRef<Path>, mut head: Parts) -> Result<Self> {
        let mut file = File::open(path).await?;
        let len = file.metadata().await?.len();
        head.headers.insert(
            header::CONTENT_LENGTH,
            // SAFETY: we know for sure the the string are valid
            unsafe { HeaderValue::from_str(len.to_string().as_str()).unwrap_unchecked() },
        );
        if len < 1024 {
            let mut buf = Vec::with_capacity(len as usize);
            file.read_to_end(&mut buf).await?;
            return Ok(Self {
                // inner: http::Response::from_parts(head, buf),
                head,
                body: buf,
                file: None,
            });
        }
        Ok(Self {
            // inner: http::Response::from_parts(head, vec![]),
            head,
            body: Body::new(),
            file: Some(file),
        })
    }

    fn from_file(file: File, head: Parts) -> Self {
        Self {
            head,
            body: Body::new(),
            file: Some(file),
        }
    }

    #[inline(always)]
    pub(crate) fn guss_buf_size(&self) -> usize {
        mem::size_of_val(self.headers())
            + STATUS_LINE_SIZE
            + if self.file.is_none() {
                self.body.len()
            } else {
                for_now!(0)
            }
    }

    #[inline(always)]
    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }

    pub(crate) fn take_file(&mut self) -> Option<File> {
        std::mem::take(&mut self.file)
    }

    // pub(crate) fn write<W: StdWrite + ?Sized>(&self, writer: &mut W) -> Result<()> {
    //     // NOTE: maybe use `write_vectored`?
    //     // The header
    //     self.write_header(writer)?;
    //     // th body
    //     StdWrite::write_all(writer, &self.body)?;
    //     // StdWrite::write_all(&mutbuf, b'\0')?;

    //     // write
    //     // StdWrite::write_all(&mut writer, &writer).map_err(crate::Error::IOError)
    //     Ok(())
    // }

    #[inline]
    fn write_status_line(&self, writer: &mut (impl StdWrite + ?Sized)) -> Result<()> {
        write!(writer, "{:?} {}{CRLF}", self.version(), self.status())
            .map_err(crate::Error::IOError)
    }

    #[inline]
    fn write_headers(&self, writer: &mut (impl StdWrite + ?Sized)) -> Result<()> {
        for (header, val) in self.headers().iter() {
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

    pub(crate) fn write_header(&self, writer: &mut (impl StdWrite + ?Sized)) -> Result<()> {
        // NOTE: maybe use `write_vectored`?
        // The header
        self.write_status_line(writer)?;
        self.write_headers(writer)?;

        // The body
        StdWrite::write_all(writer, CRLF.as_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.head.headers
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.head.headers
    }

    pub fn add_header(&mut self, header: HeaderPair) -> Option<HeaderValue> {
        self.headers_mut().insert(header.name, header.value)
    }

    #[inline]
    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.head.version
    }

    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }
}

// impl From<http::Response<Vec<u8>>> for Response {
//     #[inline(always)]
//     fn from(inner: http::Response<Vec<u8>>) -> Self {
//         Self { inner }
//     }
// }

// impl<T> From<T> for Response
// where
//     T: Deref<Target = str>,
// {
//     #[inline(always)]
//     fn from(body: T) -> Self {
//         Self::builder()
//             .body(body.as_bytes().to_vec())
//             .expect("Unreachable")
//     }
// }

const HTML_SIG: &str = "<!DOCTYPE";

impl From<Vec<u8>> for Response {
    fn from(body: Vec<u8>) -> Self {
        Self::builder()
            .header(header::CONTENT_LENGTH, body.len())
            .body(body)
            .expect("Unreachable")
    }
}

impl From<&str> for Response {
    fn from(body: &str) -> Self {
        let c_type = ContentType(if body.starts_with(HTML_SIG) {
            mime::TEXT_HTML_UTF_8
        } else {
            mime::TEXT_PLAIN_UTF_8
        });
        let body = body.as_bytes().to_vec();
        Self::builder()
            .header(header::CONTENT_LENGTH, body.len())
            .header_pair(c_type)
            .body(body)
            .expect("Unreachable")
    }
}

impl From<&[u8]> for Response {
    fn from(value: &[u8]) -> Self {
        Self::builder()
            .header(header::CONTENT_LENGTH, value.len())
            .body(value.to_vec())
            .expect("Unreachable")
    }
}
