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

impl<'a> Deref for Path<'a> {
    type Target = [&'a str];

    fn deref(&self) -> &Self::Target {
        self.chunks.deref()
    }
}

impl From<Path<'_>> for PathBuf {
    fn from(value: Path) -> Self {
        // SAFETY: the error is `Infallible`
        unsafe { PathBuf::from_str(value.as_str()).unwrap_unchecked() }
    }
}

impl Display for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Index<usize> for Path<'_> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.chunks.index(index)
    }
}

impl<'a> AsRef<std::path::Path> for Path<'a> {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self.as_str())
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
