use std::error::Error;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ClientError {
    InvalidMethod,
    InvalidParams,
    ConnectionError,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClientError::InvalidMethod => write!(f, "Requested method does not exist"),
            ClientError::InvalidParams => write!(f, "Invalid params provided"),
            ClientError::ConnectionError => write!(f, "Connection error"),
        }
    }
}

impl Error for ClientError {}
