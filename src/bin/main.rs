use std::ops::Deref;

use servi::preload::*;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(("127.0.0.1", 6173)).await.unwrap();

    let mut server = Server::new(MyRouter, listener);
    println!("starting the server");
    server.start().await.unwrap();
}

struct MyRouter;

impl Router for MyRouter {
    async fn route(&self, mut ctx: servi::Context) -> servi::Result<Response> {
        let path = ctx.path();
        match path.deref() {
            ["/"] => Ok(include_bytes!("../../pub/index.html").to_vec().into()),
            ["login"] | ["login.html"] => resource!("pub/login.html", mime::TEXT_HTML_UTF_8),
            [.., "style", _] => resource!(format!("pub/{path}"), mime::TEXT_CSS_UTF_8),
            ["ohayou"] => Ok("おはよう!".into()),
            ["echo", what] => Ok((*what).into()),
            ["echo"] => {
                drop(path); // yup, c
                let mut buf = Vec::with_capacity(512);
                ctx.request.read_body(&mut buf).await?;
                Ok(buf.into())
            }
            ["static", ..] => resource!(path, mime::IMAGE_GIF),
            ["json"] => json!({
                "code": 200,
                "success": true,
                "payload": {
                    "features": [
                        "serde",
                        "json"
                    ],
                    "homepage": null
                }
            }),
            _ => not_found!("Oh, what this even mean??"),
        }
    }
}
