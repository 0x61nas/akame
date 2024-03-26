use std::net::SocketAddr;

use http::{HeaderMap, HeaderName, Method};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

use crate::{
    request::{path::Path, query::Query, RequestLine},
    Error, Request, Result, CRLF,
};

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

    #[inline(always)]
    pub fn path(&self) -> Path {
        self.req_line.path()
    }

    #[inline(always)]
    pub fn method(&self) -> Method {
        self.req_line.method.clone()
    }

    pub fn query_params(&self) -> Option<Query> {
        self.req_line.query()
    }

    pub async fn into_request(self) -> Result<Request> {
        let (req_line, mut reader, _) = self.into_parts();
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
}
