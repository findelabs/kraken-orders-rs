use crate::errors::*;
use chrono::offset::Utc;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, to_value, Value};
use sha2::{Digest, Sha256, Sha512};
use std::error::Error;
use core::time::Duration;


pub const API_URL: &str = "https://api.kraken.com";
pub const API_VER: &str = "0";

pub struct KrakenClient {
    client: reqwest::Client,
    last_request: i64,
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl<'k> KrakenClient {
    pub fn new(api_key: &'k str, api_secret: &'k str) -> Self {

        let client = reqwest::Client::builder()
            .timeout(Duration::new(10, 0))
            .build()
            .expect("Failed to build client");

        KrakenClient {
            client,
            last_request: 0,
            api_key: Some(api_key.to_string()),
            api_secret: Some(api_secret.to_string()),
        }
    }

    pub fn signature(
        &self,
        path: &str,
        nonce: u64,
        payload: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {

        // Get message payload
        let message = format!("{}{}", nonce, &payload);

        // Get hash of message
        let hash_digest = Sha256::digest(message.as_bytes());

        // Get the private key
        let private_key = self.api_secret.as_ref().expect("Failed to get api_secret");

        // Decode private key
        let private_key_decoded = base64::decode(private_key)?;

        // Create hmac with private_key
        let mut mac = Hmac::<Sha512>::new_from_slice(&private_key_decoded).expect("here");

        // Create data from path
        let mut hmac_data = path.to_string().into_bytes();

        // Add payload to hmac
        hmac_data.append(&mut hash_digest.to_vec());

        // Add payload to mac
        mac.update(&hmac_data);

        // Encode final payload
        let b64 = base64::encode(mac.finalize().into_bytes());

        // Return base64 string
        Ok(b64)
    }

    pub async fn headers(&self, sig: &str) -> Result<HeaderMap, KrakenError> {
        // Create HeaderMap
        let mut headers = HeaderMap::new();

        // Get api key
        let api_key =
            match HeaderValue::from_str(self.api_key.as_ref().expect("Failed unwraping api_key")) {
                Ok(h) => Ok(h),
                Err(_) => Err(KrakenError::HeaderError),
            };

        // Add signature to headermap
        let api_sign = match HeaderValue::from_str(&sig) {
            Ok(h) => Ok(h),
            Err(_) => Err(KrakenError::HeaderError),
        };

        // Add all headers
        headers.insert("API-Key", api_key?);
        headers.insert("API-Sign", api_sign?);
        headers.insert(USER_AGENT, HeaderValue::from_str("kraken-rs").unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
        );

        // Return headers
        Ok(headers)
    }

    pub async fn post(&self, url: String, sig: String, payload: String) -> Result<String, KrakenError> {

        let headers = self.headers(&sig).await?;

        let client = self.client
            .post(url)
            .headers(headers)
            .body(payload);

        match client.send().await {
            Ok(m) => match m.status().as_u16() {
                429 => {
                    let text = m.text().await.expect("failed");
                    Err(KrakenError::RequestError(text))
                }
                200 => {
                    let body = match m.json::<Value>().await {
                        Ok(b) => b,
                        Err(_) => {
                            return Err(KrakenError::BadBody)
                        }
                    };

                    // Check to see if error field is empty or not
                    match &body["error"].as_array().expect("Missing error field").len() {
                        0 => {
                            log::info!("Got 200, body: {}", body.to_string());
                            Ok(body["result"].to_string())
                        },
                        _ => Err(KrakenError::RequestError(body.to_string()))
                    }
                }
                _ => Ok("Got weird result".to_owned()),
            },
            Err(e) => {
                log::error!("Caught error posting: {}", e);
                return Err(KrakenError::PostError);
            }
        }
    }

    pub async fn private(&self, path: &str, payload: Option<Value>) -> Result<String, KrakenError> {
        // Error if api_key or api_secret is missing
        if self.api_key.is_none() {
            return Err(KrakenError::ApiKey);
        } else if self.api_secret.is_none() {
            return Err(KrakenError::ApiSecret);
        };

        // Insert nonce into data
        let nonce = Utc::now().timestamp_millis() as u64;
        let payload = match payload {
            Some(mut p) => {
                let payload = p.as_object_mut().unwrap();
                payload.insert(String::from("nonce"), json!(nonce.to_string()));
                to_value(payload).expect("Failed converting Map to Value")
            }
            None => json!({"nonce": nonce.to_string()}),
        };

        // Create body as string
        let body = match serde_urlencoded::to_string(&payload) {
            Ok(b) => b,
            Err(_) => return Err(KrakenError::JsonError),
        };

        // Get signature of payload
        let path = format!("/{}/private/{}", API_VER, path);
        let url = format!("{}{}", API_URL, &path);
        let sig = match self.signature(&path, nonce, &body) {
            Ok(s) => s,
            Err(_) => return Err(KrakenError::Signature),
        };

        Ok(self.post(url, sig, body).await?)
    }

    #[allow(dead_code)]
    pub async fn add_order(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("AddOrder", Some(payload)).await?)
    }

    #[allow(dead_code)]
    pub async fn balance(&self) -> Result<String, KrakenError> {
        Ok(self.private("Balance", None).await?)
    }

    #[allow(dead_code)]
    pub async fn trade_balance(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("TradeBalance", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn open_orders(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("OpenOrders", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn closed_orders(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("ClosedOrders", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn query_orders(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("QueryOrders", Some(payload)).await?)
    }

    #[allow(dead_code)]
    pub async fn trades_history(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("TradesHistory", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn query_trades(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("QueryTrades", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn open_positions(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("OpenPositions", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn ledgers(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("Ledgers", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn query_ledgers(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("QueryLedgers", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn trade_volume(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("TradeVolume", payload).await?)
    }

    #[allow(dead_code)]
    pub async fn add_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("AddExport", Some(payload)).await?)
    }

    #[allow(dead_code)]
    pub async fn export_status(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("ExportStatus", Some(payload)).await?)
    }

    #[allow(dead_code)]
    pub async fn retrieve_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("RetrieveExport", Some(payload)).await?)
    }

    #[allow(dead_code)]
    pub async fn remove_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("RemoveExport", Some(payload)).await?)
    }
}
