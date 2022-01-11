//use serde_json::error::Error as SerdeError;
use std::fmt;
use axum::{
    body::Body,
    body::{self, Bytes},
    http::StatusCode,
    response::{IntoResponse, Response},
};


// https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html

#[derive(Debug)]
pub enum KrakenError {
    ApiKey,
    ApiSecret,
    Signature,
    PostError,
    JsonError,
    HeaderError,
    BadBody,
    RequestError(String)
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
            KrakenError::RequestError(ref e) => f.write_str(e)
        }
    }
}

impl IntoResponse for KrakenError {
    fn into_response(self) -> Response {
        let payload = self.to_string();
        let body = body::boxed(body::Full::from(payload));

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}


//impl std::convert::From<reqwest::Error> for KrakenError {
//    fn from(e: reqwest::Error) -> Self {
//        KrakenError::ReqwestError(e)
//    }
//}

// Example of how to wrap errors into krakenerror
// https://docs.rs/backoff/0.3.0/src/backoff/error.rs.html#9-16
