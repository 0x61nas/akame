use std::ops::Deref;

#[derive(Debug, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct Path<'inner> {
    inner: &'inner str,
}

impl<'inner> Path<'inner> {
    pub const fn as_str(&self) -> &str {
        self.inner
    }

    pub(crate) const fn new(str: &'inner str) -> Path<'inner> {
        Path { inner: str }
    }

    pub fn chunks(&self) -> Box<[&str]> {
        if self.inner.len() == 1 {
            return Box::new([self.inner]);
        }
        self.inner
            .split('/')
            .skip(1)
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .into_boxed_slice()
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
