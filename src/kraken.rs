use crate::errors::*;
use chrono::offset::Utc;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, to_value, Value};
use sha2::{Digest, Sha256, Sha512};
use std::error::Error;
use core::time::Duration;
use std::convert::Infallible;
use serde::Deserialize;

pub const API_URL: &str = "https://api.kraken.com";
pub const API_VER: &str = "0";

pub enum Payloads {
    TradeBalance,
    OpenOrders
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TradeBalance {
    asset: String
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenOrders {
    trades: Option<bool>,
    userref: Option<u32>
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KrakenClient {
    client: reqwest::Client,
    last_request: i64,
    api_key: String,
    private_key: String,
}

#[derive(Debug, Clone, Default)]
pub struct KrakenBuilder {
    api_key: String,
    private_key: String
}

impl KrakenBuilder {
    pub fn api_key(mut self, arg: &str) -> Self {
        self.api_key = arg.to_string();
        self
    }

    pub fn private_key(mut self, arg: &str) -> Self {
        self.private_key = arg.to_string();
        self
    }

    pub fn build(&mut self) -> Result<KrakenClient, Infallible> {
        let client = reqwest::Client::builder()
            .timeout(Duration::new(10, 0))
            .build()
            .expect("Failed to build client");

        Ok(KrakenClient {
            client,
            last_request: 0,
            api_key: self.api_key.to_string(),
            private_key: self.private_key.to_string(),
        })
    }
}

impl<'a, 'k> KrakenClient {
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

        // Decode private key
        let private_key_decoded = base64::decode(&self.private_key)?;

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

    pub async fn parse(&self, p: Payloads, payload: Option<&'a Value>) -> Result<Option<&'a Value>, KrakenError> {
        use Payloads::*;
        let l = match payload {
            Some(l) => l,
            None => return Ok(None)
        };

        match p {
            TradeBalance => {
                let _: crate::kraken::TradeBalance = serde_json::from_value(l.clone())?;
                Ok(payload)
            },
            OpenOrders => {
                let _: crate::kraken::OpenOrders = serde_json::from_value(l.clone())?;
                Ok(payload)
            }
        }
    }

    pub async fn headers(&self, sig: Option<String>) -> Result<HeaderMap, KrakenError> {
        // Create HeaderMap
        let mut headers = HeaderMap::new();

        // Get api key
        let api_key =
            match HeaderValue::from_str(&self.api_key) {
                Ok(h) => Ok(h),
                Err(_) => Err(KrakenError::HeaderError),
            };

        // Add signature to headermap
        if let Some(sig) = sig {
            match HeaderValue::from_str(&sig) {
                Ok(h) => headers.insert("API-Sign", h),
                Err(_) => return Err(KrakenError::HeaderError),
            };
        };

        // Add all headers
        headers.insert("API-Key", api_key?);
        headers.insert(USER_AGENT, HeaderValue::from_str("kraken-rs").unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
        );

        // Return headers
        Ok(headers)
    }

    pub async fn post(&self, url: String, sig: Option<String>, payload: String) -> Result<String, KrakenError> {

        let headers = self.headers(sig).await?;

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
                            log::debug!("Got 200, body: {}", body.to_string());
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

    pub async fn public(&self, path: &str, payload: Option<Value>) -> Result<String, KrakenError> {

        // Create body as string
        let body = match serde_urlencoded::to_string(&payload) {
            Ok(b) => b,
            Err(_) => return Err(KrakenError::JsonError),
        };

        // Get signature of payload
        let path = format!("/{}/public/{}", API_VER, path);
        let url = format!("{}{}", API_URL, &path);
        Ok(self.post(url, None, body).await?)
    }

    pub async fn private(&self, path: &str, payload: Option<Value>) -> Result<String, KrakenError> {

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

        Ok(self.post(url, Some(sig), body).await?)
    }

    pub async fn balance(&self) -> Result<String, KrakenError> {
        Ok(self.private("Balance", None).await?)
    }

    pub async fn trade_balance(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        self.parse(Payloads::TradeBalance, payload.as_ref()).await?;
        Ok(self.private("TradeBalance", payload).await?)
    }

    pub async fn open_orders(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        self.parse(Payloads::OpenOrders, payload.as_ref()).await?;
        Ok(self.private("OpenOrders", payload).await?)
    }

    pub async fn closed_orders(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("ClosedOrders", payload).await?)
    }

    pub async fn query_orders(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("QueryOrders", Some(payload)).await?)
    }

    pub async fn trades_history(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("TradesHistory", payload).await?)
    }

    pub async fn query_trades(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("QueryTrades", payload).await?)
    }

    pub async fn open_positions(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("OpenPositions", payload).await?)
    }

    pub async fn ledgers(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("Ledgers", payload).await?)
    }

    pub async fn query_ledgers(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("QueryLedgers", payload).await?)
    }

    pub async fn trade_volume(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.private("TradeVolume", payload).await?)
    }

    pub async fn add_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("AddExport", Some(payload)).await?)
    }

    pub async fn export_status(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("ExportStatus", Some(payload)).await?)
    }

    pub async fn retrieve_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("RetrieveExport", Some(payload)).await?)
    }

    pub async fn remove_export(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("RemoveExport", Some(payload)).await?)
    }

    // Public Endpoints
    pub async fn assets(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.public("Assets", payload).await?)
    }
    
    pub async fn asset_pairs(&self, payload: Option<Value>) -> Result<String, KrakenError> {
        Ok(self.public("AssetPairs", payload).await?)
    }
    
    pub async fn ticker(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.public("Ticker", Some(payload)).await?)
    }
        
    pub async fn ohlc(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.public("OHLC", Some(payload)).await?)
    }

    pub async fn depth(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.public("Depth", Some(payload)).await?)
    }

    pub async fn trades(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.public("Trades", Some(payload)).await?)
    }

    pub async fn spread(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.public("Spread", Some(payload)).await?)
    }

    pub async fn add_order(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("AddOrder", Some(payload)).await?)
    }

    pub async fn cancel_order(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("CancelOrder", Some(payload)).await?)
    }

    pub async fn cancel_all(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("CancelAll", Some(payload)).await?)
    }

    pub async fn cancel_all_orders_after(&self, payload: Value) -> Result<String, KrakenError> {
        Ok(self.private("CancelAllOrdersAfter", Some(payload)).await?)
    }
}
