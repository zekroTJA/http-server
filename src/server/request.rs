use core::fmt;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    path::PathBuf,
    vec,
};

#[derive(Debug)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Custom(String),
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Trace => write!(f, "TRACE"),
            Self::Custom(v) => write!(f, "{}", v.to_uppercase()),
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "CONNECT" => Self::Connect,
            "OPTIONS" => Self::Options,
            "TRACE" => Self::Trace,
            _ => Self::Custom(value.to_string()),
        }
    }
}

#[derive(Default, Debug)]
pub struct HeaderMap(HashMap<String, RefCell<Vec<String>>>);

impl HeaderMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<K: AsRef<str>, V: Into<String>>(&mut self, key: K, value: V) {
        let key: String = canonicalize(key.as_ref());
        let entry = self.0.entry(key).or_insert(RefCell::new(vec![]));
        entry.borrow_mut().push(value.into());
    }

    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<Ref<Vec<String>>> {
        let key = canonicalize(key.as_ref());
        self.0.get(&key).map(|v| v.borrow())
    }
}

fn canonicalize(key: &str) -> String {
    let mut upper = true;
    let mut new = String::with_capacity(key.len());

    for c in key.trim().chars() {
        if c == '-' {
            upper = true;
            new.push(c);
            continue;
        }

        if upper {
            if !c.is_uppercase() {
                new.push(c.to_uppercase().next().unwrap());
            } else {
                new.push(c);
            }
            upper = false;
            continue;
        }

        if c.is_uppercase() {
            new.push(c.to_lowercase().next().unwrap());
        } else {
            new.push(c);
        }
    }

    new
}

impl IntoIterator for HeaderMap {
    type Item = (String, String);
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut res = vec![];

        for (k, vs) in self.0.iter() {
            for v in vs.borrow().iter() {
                res.push((k.to_string(), v.to_string()));
            }
        }

        res.into_iter()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Request {
    pub method: Method,
    pub proto: String,
    pub path: PathBuf,
    pub header: HeaderMap,
    pub body: Option<Vec<u8>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn canonicalize_test() {
        assert_eq!(canonicalize("content-type"), "Content-Type");
        assert_eq!(canonicalize("Content-Type"), "Content-Type");
        assert_eq!(canonicalize("CONTENT-type"), "Content-Type");
    }
}
