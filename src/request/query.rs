use std::{collections::HashMap, str::Split};

pub type QueryItem<'a> = (&'a str, &'a str);

#[derive(Debug, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Query<'inner> {
    inner: &'inner str,
}

impl<'inner> Query<'inner> {
    pub(crate) const fn new(inner: &'inner str) -> Self {
        Self { inner }
    }

    pub const fn from_static(inner: &'static str) -> Self {
        Self::new(inner)
    }
}

pub struct QueryIterator<'inner> {
    inner: Split<'inner, char>,
}

impl<'inner> QueryIterator<'inner> {
    pub fn new(query: Query<'inner>) -> Self {
        Self {
            inner: query.inner.split('&'),
        }
    }
}

impl<'inner> IntoIterator for Query<'inner> {
    type Item = QueryItem<'inner>;

    type IntoIter = QueryIterator<'inner>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIterator::new(self)
    }
}

impl<'inner> Iterator for QueryIterator<'inner> {
    type Item = QueryItem<'inner>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        next.split_once('=')
    }
}

impl<'a> From<Query<'a>> for HashMap<&'a str, &'a str> {
    fn from(value: Query<'a>) -> Self {
        HashMap::from_iter(value)
    }
}
