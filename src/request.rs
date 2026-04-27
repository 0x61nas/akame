pub mod path;
pub mod query;

use std::marker;

use http::{HeaderMap, HeaderName, Method, Uri};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::tcp::OwnedReadHalf,
};

use crate::{Error, CRLF};

use self::{path::Path, query::Query};

pub type Body = BufReader<OwnedReadHalf>;

#[derive(Debug)]
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

    pub(crate) async fn from_row(mut reader: BufReader<OwnedReadHalf>) -> crate::Result<Self> {
        let req_line = RequestLine::from_row(&mut reader).await?;
        let mut headers = HeaderMap::new();
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
            headers.insert(
                HeaderName::from_bytes(key.as_bytes()).expect("Invalid header name"),
                value.trim().parse().expect("Invalid header value"),
            );
        }
        // let mut body = Vec::new();
        // // body
        // if let Some(len) = rb.headers_ref().unwrap().get(header::CONTENT_LENGTH) {
        //     let mut buffer = Vec::with_capacity(len.to_str().unwrap().parse().unwrap());
        //     while let Ok(n) = reader.read(&mut buffer).await {
        //         if n == 0 {
        //             break;
        //         }
        //         body.extend(&buffer);
        //     }
        // }

        // rb.body(body).map_err(|_| crate::Error::InvalidRequest)
        Ok(Request::new(req_line, headers, reader))
    }

    pub async fn read_body(
        &mut self,
        writer: &mut (impl AsyncWrite + marker::Unpin),
    ) -> crate::Result<usize> {
        let Some(len) = self.headers.get(crate::header::CONTENT_LENGTH) else {
            todo!()
        };
        let len = len.to_str().unwrap().parse::<usize>().unwrap();
        let mut buffer = Vec::with_capacity(len);
        // let mut body = self.body.clone();
        while let Ok(n) = self.body.read_buf(&mut buffer).await {
            writer.write_all(&buffer).await?;
            if n == len {
                break;
            }
        }
        Ok(len)
    }

    pub fn path(&self) -> Path<'_> {
        self.req_line.path()
    }
}

#[derive(Debug, PartialEq, Hash, Clone)]
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
    pub fn path(&self) -> Path<'_> {
        Path::new(self.uri.path())
    }

    #[inline]
    pub fn query(&self) -> Option<Query<'_>> {
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

    #[inline(always)]
    pub async fn from_row(reader: &mut BufReader<OwnedReadHalf>) -> crate::Result<Self> {
        let mut buf = Vec::new();
        reader.read_until(0xA, &mut buf).await?;
        if let Some(lb) = buf.pop() {
            if lb != b'\n' {
                // invalid_request(xx);
                // return Ok(());
                return Err(crate::Error::InvalidRequest);
            }
        }
        Self::from_bytes(buf)
    }
}
