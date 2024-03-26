pub mod path;
pub mod query;

use http::{HeaderMap, Method, Uri};
use tokio::{io::BufReader, net::tcp::OwnedReadHalf};

use self::{path::Path, query::Query};

pub type Body = BufReader<OwnedReadHalf>;

pub struct Request {
    pub req_line: RequestLine,
    pub headers: HeaderMap,
    pub body: Body,
}

impl Request {
    pub(crate) fn new(req_line: RequestLine, headers: HeaderMap, body: Body) -> Self {
        Self {
            req_line,
            headers,
            body,
        }
    }

    // pub fn path(&self) -> Path {
    //     Path::new(self.req_line.uri.path())
    // }
}

#[derive(Debug, PartialEq, Hash)]
pub struct RequestLine {
    pub method: Method,
    pub uri: Uri,
    pub http_version: http::Version,
}

impl RequestLine {
    pub fn new(uri: Uri) -> Self {
        Self::new_with_method(uri, Method::GET)
    }

    #[inline]
    pub fn path(&self) -> Path {
        Path::new(self.uri.path())
    }

    #[inline]
    pub fn query(&self) -> Option<Query> {
        Some(Query::new(self.uri.query()?))
    }

    #[inline]
    pub fn new_with_method(uri: Uri, method: Method) -> Self {
        Self::new_with_method_and_version(method, uri, http::Version::HTTP_11)
    }

    #[inline]
    pub fn new_with_method_and_version(
        method: Method,
        uri: Uri,
        http_version: http::Version,
    ) -> Self {
        Self {
            method,
            uri,
            http_version,
        }
    }

    pub fn from_bytes<B: IntoIterator<Item = u8>>(bytes: B) -> crate::Result<Self> {
        let mut bytes = bytes.into_iter();
        macro_rules! chunk {
            ($buf: ident) => {
                while let Some(b) = bytes.next() {
                    if b == b' ' {
                        break;
                    }
                    $buf.push(b);
                }
            };
            ($buf: expr) => {{
                let mut buf = $buf;
                chunk!(buf);
                buf
            }};
        }

        let method = Method::from_bytes(&chunk!(Vec::with_capacity(7)))?;
        let uri = Uri::try_from(chunk!(Vec::new())).expect("Invalid URI");
        let http_version = {
            use http::Version;
            let version = chunk!(Vec::with_capacity(8));
            match &version[..] {
                b"HTTP/1.1" | b"HTTP/1" => Version::HTTP_11,
                b"HTTP/2.0" | b"HTTP/2" => Version::HTTP_2,
                b"HTTP/3.0" | b"HTTP/3" => Version::HTTP_3,
                _ => Version::HTTP_11,
            }
        };

        Ok(Self::new_with_method_and_version(method, uri, http_version))
    }
}
