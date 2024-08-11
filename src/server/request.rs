use std::{collections::HashMap, path::PathBuf};

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

pub struct Request {
    pub method: Method,
    pub proto: String,
    pub path: PathBuf,
    pub header: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,
}
