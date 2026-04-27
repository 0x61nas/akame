use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use tokio::{io::BufReader, net::tcp::OwnedReadHalf};

use crate::{Request, Result};

#[derive(Debug)]
#[doc(alias = "ctx")]
#[non_exhaustive]
pub struct Context {
    pub request: Request,
    pub addr: SocketAddr,
}

impl Context {
    #[inline(always)]
    pub(crate) async fn new(reader: BufReader<OwnedReadHalf>, addr: SocketAddr) -> Result<Self> {
        Ok(Self {
            request: Request::from_row(reader).await?,
            addr,
        })
    }

    pub fn into_parts(self) -> (Request, SocketAddr) {
        (self.request, self.addr)
    }
}

impl Deref for Context {
    type Target = Request;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}
