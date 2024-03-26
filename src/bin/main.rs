use std::ops::Deref;

use http::header;
use servi::{not_found, Response, Router, Server};
use tokio::{io::AsyncReadExt, net::TcpListener};

#[tokio::main]
async fn main() {
    //
    let listener = TcpListener::bind(("127.0.0.1", 6173)).await.unwrap();

    let mut server = Server::new(MyRouter, listener);
    println!("starting the server");
    server.start().await.unwrap();

    println!("test");
}

struct MyRouter;

impl Router for MyRouter {
    async fn route(&self, ctx: servi::Context) -> servi::Result<Response> {
        // let request = ctx.into_request().await?;
        // let _ = dbg!(ctx.into_request().await);
        let ctx = dbg!(&ctx);
        match dbg!(ctx.path().chunks()).deref() {
            // ["/"] => Response::builder()
            //     .body(b"Ohayou!".iter().map(|b| b.to_owned()).collect::<Vec<u8>>()),
            ["/"] => Ok(include_bytes!("../../index.html").to_vec().into()),
            ["ohayou"] => Ok("おはよう!".into()),
            ["echo", what] => Ok(what.as_bytes().into()),
            // ["/", "echo"] => {
            //     let mut request = ctx.into_request().await?;
            //     let mut buf = Vec::with_capacity(
            //         request
            //             .headers
            //             .get(header::CONTENT_LENGTH)
            //             .map(|v| v.to_str().unwrap().parse::<usize>().unwrap())
            //             .unwrap_or(512),
            //     );
            //     request.body.read_to_end(&mut buf).await?;
            //     Ok(buf.into())
            // }
            _ => not_found!("OOPS, 404 -\\()/-".as_bytes().to_vec()),
        }
    }
}
