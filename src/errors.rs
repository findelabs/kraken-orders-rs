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
    BadBody
}

impl std::error::Error for KrakenError {}

impl fmt::Display for KrakenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KrakenError::ApiKey => f.write_str("Missing API Key"),
            KrakenError::ApiSecret => f.write_str("Missing API Secret"),
            KrakenError::Signature=> f.write_str("Error generating signature"),
            KrakenError::PostError=> f.write_str("Error posting to Kraken"),
            KrakenError::JsonError=> f.write_str("Error converting payload to string"),
            KrakenError::HeaderError=> f.write_str("Error generating headers for request"),
            KrakenError::BadBody=> f.write_str("Could not unpack response body")
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
