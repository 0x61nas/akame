use servi::{not_found, Response, Router, Server, StatusCode};
use tokio::net::TcpListener;

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
        let path = String::from_utf8(ctx.req_line.path.clone()).unwrap();
        // let _ = dbg!(ctx.into_request().await);
        match path.as_str() {
            "/" => Response::builder()
                .body(b"Ohayou!".iter().map(|b| b.to_owned()).collect::<Vec<u8>>()),
            "/ohayou" => Ok("おはよう!".into()),
            _ => not_found!(),
        }
    }
}
