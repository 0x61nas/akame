use std::{
    fmt::Display,
    hash::Hash,
    ops::{Deref, Index},
    path::PathBuf,
    str::FromStr,
};

use smallvec::{smallvec, SmallVec};

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct Path<'inner> {
    inner: &'inner str,
    chunks: SmallVec<[&'inner str; 6]>,
}

impl<'inner> Path<'inner> {
    pub const fn as_str(&self) -> &str {
        self.inner
    }

    pub(crate) fn new(str: &'inner str) -> Path<'inner> {
        let str = &str[1..];
        Path {
            inner: str,
            chunks: Self::_chunks(str),
        }
    }

    fn _chunks(str: &str) -> SmallVec<[&str; 6]> {
        if str.len() <= 1 {
            return smallvec!["/"];
        }
        str.split('/')
            // .skip(1)
            .filter(|p| !p.is_empty())
            .collect::<SmallVec<_>>()
    }
}

impl Deref for Path<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

// #[macro_export]
// macro_rules! path {
//     (/) => {
//         [""]
//     };
//     ($endpoint: expr) => {
//         $crate::path!(/$endpoint)
//     };
//     (/$endpoint: expr) => {
//         ["", $endpoint]
//     };
//     ($($endpoint: item/)+) => {
//         ["", stringfy!($($endpoint),+)]
//     };
// }
