use bytes::Bytes;
use hyper::Client;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;


//type HttpsClient = Client<HttpsConnector<HttpConnector>, hyper::Body>;
pub type HyperClient = Client<HttpsConnector<hyper::client::HttpConnector>>;

pub struct Kraken {
    last_request: i64,
    api_secret: String,
    https_client: HyperClient
}

impl<'k> Kraken {
    pub fn new(api_secret: &'k str) -> Self {
        let https_client = Client::builder().build(HttpsConnector::new());

        Kraken {
            last_request: 0,
            api_secret: api_secret.to_string(),
            https_client
        }
    }
}
