use std::net::SocketAddr;

use http::{header, Method};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

use crate::{Error, Request, Result, CRLF};

#[derive(Debug)]
pub struct Context {
    pub req_line: RequestLine,
    reader: BufReader<OwnedReadHalf>,
    pub addr: SocketAddr,
}

impl Context {
    pub(crate) fn new(
        req_line: RequestLine,
        reader: BufReader<OwnedReadHalf>,
        addr: SocketAddr,
    ) -> Self {
        Self {
            req_line,
            reader,
            addr,
        }
    }

    pub fn into_parts(self) -> (RequestLine, BufReader<OwnedReadHalf>, SocketAddr) {
        (self.req_line, self.reader, self.addr)
    }

    // pub fn inner_reader(self) -> OwnedReadHalf {
    //     self.reader
    // }

    pub async fn into_request(self) -> Result<Request> {
        let (req_line, mut reader, _) = self.into_parts();
        let mut rb = http::Request::builder()
            .method(req_line.method)
            .version(req_line.http_version)
            .uri(req_line.path.clone());
        loop {
            let mut buf = Vec::with_capacity(60);
            let n = reader.read_until(b'\n', &mut buf).await?;
            // .context("Read headers")?;
            if n == 0 {
                break;
            }
            // println!("> {buf:?}");
            let buf = String::from_utf8(buf).unwrap();
            // println!("> {buf}");
            if buf == CRLF {
                break;
            }
            let Some((key, value)) = buf.split_once(':') else {
                return Err(Error::InvalidHeader(buf));
            };
            rb = rb.header(key.to_lowercase(), value.trim());
        }
        let mut body = Vec::new();
        // body
        if let Some(len) = rb.headers_ref().unwrap().get(header::CONTENT_LENGTH) {
            let mut buffer = Vec::with_capacity(len.to_str().unwrap().parse().unwrap());
            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                body.extend(&buffer);
            }
        }

        rb.body(body).map_err(|_| crate::Error::InvalidRequest)
    }
}

#[derive(Debug, PartialEq, Hash)]
#[non_exhaustive]
pub struct RequestLine {
    pub method: Method,
    pub path: Vec<u8>,
    pub http_version: http::Version,
}

impl RequestLine {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self::new_with_method(bytes, Method::GET)
    }

    #[inline]
    pub fn new_with_method(path: Vec<u8>, method: Method) -> Self {
        Self::new_with_method_and_version(method, path, http::Version::HTTP_11)
    }

    #[inline]
    pub fn new_with_method_and_version(
        method: Method,
        path: Vec<u8>,
        http_version: http::Version,
    ) -> Self {
        Self {
            method,
            path,
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
        let path = chunk!(Vec::new());
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

        Ok(Self::new_with_method_and_version(
            method,
            path,
            http_version,
        ))
    }
}
