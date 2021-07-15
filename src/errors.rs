//use serde_json::error::Error as SerdeError;
use std::fmt;

#[derive(Debug)]
pub enum KrakenError {
    ApiKey,
    ApiSecret,
    Signature,
    PostError,
    JsonError,
    HeaderError,
    BadBody,
    RequestError,
}

pub struct RequestError {
    details: String,
}

impl std::error::Error for KrakenError {}

impl fmt::Display for KrakenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KrakenError::ApiKey => f.write_str("Missing API Key"),
            KrakenError::ApiSecret => f.write_str("Missing API Secret"),
            KrakenError::Signature => f.write_str("Error generating signature"),
            KrakenError::PostError => f.write_str("Error posting to Kraken"),
            KrakenError::JsonError => f.write_str("Error converting payload to string"),
            KrakenError::HeaderError => f.write_str("Error generating headers for request"),
            KrakenError::BadBody => f.write_str("Could not unpack response body"),
            KrakenError::RequestError => write!(f, "{}", self),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl RequestError {
    pub fn new(msg: &str) -> RequestError {
        RequestError {
            details: msg.to_string(),
        }
    }
}

/*
impl From<bson::ser::Error> for MyError {
    fn from(e: bson::ser::Error) -> Self {
        match e {
            _ => MyError::BsonError,
        }
    }
}
*/

// Example of how to wrap errors into krakenerror
// https://docs.rs/backoff/0.3.0/src/backoff/error.rs.html#9-16
