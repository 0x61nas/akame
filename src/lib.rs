pub mod context;
pub mod header;
pub mod macros;
pub mod request;
pub mod response;
pub mod router;

use std::{marker::PhantomData, net::SocketAddr, sync::Arc};

use request::RequestLine;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
};

// re-exports
pub use context::Context;
pub use http::Method;
pub use http::StatusCode;
pub use request::Request;
pub use response::Response;
pub use router::Router;

const CRLF: &str = "\r\n";

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    IOError(#[from] tokio::io::Error),
    #[error("{0}")]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error("Invalid header: `{0}`")]
    InvalidHeader(String),
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Invalid response: `{0}`")]
    InvalidResponse(#[from] http::Error),
}

pub type Result<T> = std::result::Result<T, crate::Error>;

// pub type Request = http::Request<Vec<u8>>;

pub struct Server<R>
where
    R: Router + Sync + Send,
    // B: AsBytes,
{
    listener: TcpListener,
    // endpointes: HashMap<EndPoint, Box<dyn FnOnce(Request) -> Response>>,
    router: R,
}

#[non_exhaustive]
pub struct ServerConfig {
    pub response_file_buffer: usize,
}

// pub struct ServerBulider<R, B>
// where
//     R: Router<B>,
// {
//     router: R,
//     _phontom: PhantomData<B>,
// }

impl<R> Server<R>
where
    R: Router + Sync + Send + 'static,
    // B: AsBytes + Sync + Send,
{
    // pub fn bulder() -> ServerBulider<R, B> {
    //     // ServerBuilder::new()
    //     todo!()
    // }
    pub fn new(router: R, listener: TcpListener) -> Arc<Self> {
        Arc::new(Self { listener, router })
    }

    pub async fn start(self: &mut Arc<Self>) -> Result<()> {
        while let Ok((stream, addr)) = self.listener.accept().await {
            let server = self.clone();
            tokio::spawn(Self::handle_request(server, stream, addr));
        }
        Ok(())
    }

    async fn handle_request(self: Arc<Self>, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::new();
        reader.read_until(0xA, &mut buf).await?;
        if let Some(lb) = buf.pop() {
            if lb != b'\n' {
                invalid_request();
                return Ok(());
            }
        }
        let req_line = dbg!(RequestLine::from_bytes(buf)?);
        let ctx = Context::new(req_line, reader, addr);
        // Route
        let Ok(mut response) = self.router.route(ctx).await else {
            internal_error(writer);
            return Ok(());
        };
        let mut response = response.into();

        // write the response
        let _buf_size = response.guss_buf_size();
        let mut buf = Vec::with_capacity(_buf_size);
        if let Some(file) = response.take_file() {
            response.write_header(&mut buf)?;
            writer.write_all(&buf).await?;
            let mut buf = Vec::with_capacity(for_now!(1042));
            let mut reader = BufReader::new(file);
            loop {
                let Ok(n) = reader.read_buf(&mut buf).await else {
                    internal_error(writer);
                    return Ok(());
                };
                writer.write_all(&buf).await?;
                if n < buf.len() {
                    break;
                }
            }
        } else {
            // TODO: compress the data?
            response.write(&mut buf)?;
            writer.write_all(&buf).await?;
            debug_assert!(_buf_size >= buf.capacity());
        }
        // writer.flush().await?;
        // writer.shutdown().await?;
        Ok(())
    }
}

fn invalid_request() {
    todo!()
}

fn internal_error(_: OwnedWriteHalf) {
    todo!("5xx")
}
