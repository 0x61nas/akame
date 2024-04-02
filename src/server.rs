use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::Context;
use crate::Result;
use crate::Router;

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
        let reader = BufReader::new(reader);
        let Ok(ctx) = Context::new(reader, addr).await else {
            invalid_request();
            return Ok(());
        };
        // Route
        let Ok(response) = self.router.route(ctx).await else {
            internal_error(writer);
            return Ok(());
        };
        let mut response = response.into();
        // Write the respoonse HTTP header
        let mut buf = Vec::with_capacity(response.guss_buf_size());
        response.write_header(&mut buf)?;
        writer.write_all(&buf).await?;

        if let Some(file) = response.take_file() {
            const CAP: usize = 7024;
            let mut buf = [0; CAP];
            let mut reader = BufReader::new(file);
            loop {
                let Ok(n) = reader.read(&mut buf).await else {
                    internal_error(writer);
                    return Ok(());
                };
                if n == 0 {
                    break;
                }
                writer.write_all(&buf[..n]).await?;
            }
        } else {
            // TODO: compress the data?
            writer.write_all(&response.body).await?;
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
