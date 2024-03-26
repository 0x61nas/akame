use std::future::Future;

use crate::{Context, Response, Result};

pub trait Router {
    fn route(
        &self,
        ctx: Context,
    ) -> impl Future<Output = Result<impl Into<Response>>> + Sync + Send;
}

// pub trait RequestHandler<B> {
//     async fn handle(self: &mut Self, ctx: Context) -> Response<B>;
// }

// pub struct SimpleRouter<'h, B: From<Vec<u8>>> {
//     map: HashMap<Box<[u8]>, Box<&'h dyn RequestHandler<B>>>,
// }
