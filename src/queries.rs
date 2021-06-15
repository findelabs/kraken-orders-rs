use std::error;
use sha2::{Sha256, Sha512, Digest};
use std::error::Error;
use hmac::{Hmac, Mac, NewMac};
use chrono::offset::Utc;


type BoxResult<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

#[derive(Default, Debug, Clone)]
struct Query<'a> {
    name: &'a str,
    required: bool,
    value: Option<&'a str>
}

#[derive(Debug, Clone, Copy)]
struct Nonce {
    time: i64
}

#[derive(Default, Debug)]
pub struct RequestQueries<'a> {
    nonce: Nonce,          // int, ever-increasing u64
    userref: Query<'a>,    // int, user reference id
    ordertype: Query<'a>,  // string, market, limit, stop-loss, take-profit, stop-loss-limit, take-profit-limit, settle-position
    r#type: Query<'a>,     // string, buy/sell
    volume: Query<'a>,     // Order quantity in terms of the base asset
    pair: Query<'a>,       // Crypto pair
    price: Query<'a>,      // Limit price for limit orders, Trigger price for stop-loss, stop-loss-limit, take-profit and take-profit-limit orders
    price2: Query<'a>,     // Limit price for stop-loss-limit and take-profit-limit orders
    leverage: Query<'a>,   // Amount of leverage desired (default = none)
    oflags: Query<'a>,     // Comma delimited list of order flags
    starttm: Query<'a>,    // string, Scheduled start time
    expiretm: Query<'a>,   // string, Expiration time
    validate: Query<'a>    // Validate inputs only. Do not submit order
}

impl Default for Nonce {
    fn default() -> Self { 
        let time = Utc::now().timestamp_nanos();
        Self { time }
    }
}

impl<'b> RequestQueries<'b> {
    pub fn signature(&self, path: &str, private_key: &str) -> Result<String,Box<dyn Error + Send + Sync>> {
        let postdata = self.post_data().unwrap();
        let encoded = format!("{}{}", self.nonce(), &postdata);
    
        let mut hasher = Sha256::new();
        hasher.update(encoded);
        let result = hasher.finalize();
    
        let mut message = path.as_bytes().to_vec();
        for elem in result {
            message.push(elem);
        }
    
        let hmac_key = base64::decode(private_key.as_bytes())?;
        let mut mac = Hmac::<Sha512>::new_from_slice(&hmac_key).expect("here");
        mac.update(&message);
        let mac_result = mac.finalize();
    
        let b64 = base64::encode(&mac_result.into_bytes());
    
        Ok(b64)
    }

    pub fn new() -> RequestQueries<'static> { 
        RequestQueries {
            nonce: Nonce::default(), 
            userref: Query {name: "userref", required: false, value: None},
            ordertype: Query {name: "ordertype", required: true, value: None},
            r#type: Query {name: "type", required: true, value: None},
            volume: Query {name: "volume", required: false, value: None},
            pair: Query {name: "pair", required: true, value: None},
            price: Query {name: "price", required: false, value: None},
            price2: Query {name: "price2", required: false, value: None},
            leverage: Query {name: "leverage", required: false, value: None},
            oflags: Query {name: "oflags", required: false, value: None},
            starttm: Query {name: "starttm", required: false, value: None},
            expiretm: Query {name: "expiretm", required: false, value: None},
            validate: Query {name: "validate", required: false, value: None}
        }
    }

    pub fn post_data(&self) -> Option<String> {
        let queries: [&str; 12] = [
            "ordertype",
            "pair",
            "price",
            "r#type",
            "volume",
            "userref",
            "price2",
            "leverage",
            "oflags",
            "starttm",
            "expiretm",
            "validate"
        ];

        let mut query_pairs = String::new();

        for query in queries.iter() {
            let spacer = if &query_pairs.len() > &0 {"&" } else {""};
            query_pairs.push_str(&format!("{}nonce={}", spacer, self.nonce()));
            match self.get(query) {
                Some(item) => {
                    match item.value {
                        Some(data) => {
                            query_pairs.push_str(&format!("{}{}={}", spacer, &item.name, &data));
                        },
                        None => continue
                    }
                }
                None => continue,
                _ => continue
            };
        };
    
        Some(query_pairs)
    }

    pub fn get<'a>(&self, field: &str) -> Option<Query> {
        match field {
            "userref" => Some(self.userref.clone()),
            "ordertype" => Some(self.ordertype.clone()),
            "r#type" => Some(self.r#type.clone()),
            "volume" => Some(self.volume.clone()),
            "pair" => Some(self.pair.clone()),
            "price" => Some(self.price.clone()),
            "price2" => Some(self.price2.clone()),
            "leverage" => Some(self.leverage.clone()),
            "oflags" => Some(self.oflags.clone()),
            "starttm" => Some(self.starttm.clone()),
            "expiretm" => Some(self.expiretm.clone()),
            "validate" => Some(self.validate.clone()),
            _ => None
        }
    }

    pub fn set_userref<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.userref.value = Some(value);
        self
    }

    pub fn set_ordertype<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.ordertype.value = Some(value);
        self
    }

    pub fn set_type<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.r#type.value = Some(value);
        self
    }

    pub fn set_volume<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.volume.value = Some(value);
        self
    }

    pub fn set_pair<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.pair.value = Some(value);
        self
    }

    pub fn set_price<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.price.value = Some(value);
        self
    }

    pub fn set_price2<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.price2.value = Some(value);
        self
    }

    pub fn set_leverage<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.leverage.value = Some(value);
        self
    }

    pub fn set_oflags<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.oflags.value = Some(value);
        self
    }

    pub fn set_starttm<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.starttm.value = Some(value);
        self
    }

    pub fn set_expiretm<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.expiretm.value = Some(value);
        self
    }

    pub fn set_validate<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.validate.value = Some(value);
        self
    }

    pub fn nonce<'a>(&'a self) -> i64 {
        self.nonce.time
    }

    pub fn userref<'a>(&'a self) -> Option<&'a str> {
        self.userref.value.clone()
    }

    pub fn ordertype<'a>(&'a self) -> Option<&'a str> {
        self.ordertype.value.clone()
    }

    pub fn r#type<'a>(&'a self) -> Option<&'a str> {
        self.r#type.value.clone()
    }

    pub fn volume<'a>(&'a self) -> Option<&'a str>{
        self.volume.value.clone()
    }

    pub fn pair<'a>(&'a self) -> Option<&'a str> {
        self.pair.value.clone()
    }

    pub fn price<'a>(&'a self) -> Option<&'a str>{
        self.price.value.clone()
    }

    pub fn price2<'a>(&'a self) -> Option<&'a str>{
        self.price2.value.clone()
    }

    pub fn leverage<'a>(&'a self) -> Option<&'a str>{
        self.leverage.value.clone()
    }

    pub fn oflags<'a>(&'a self) -> Option<&'a str>{
        self.oflags.value.clone()
    }

    pub fn starttm<'a>(&'a self) -> Option<&'a str>{
        self.starttm.value.clone()
    }

    pub fn expiretm<'a>(&'a self) -> Option<&'a str>{
        self.expiretm.value.clone()
    }

    pub fn validate<'a>(&'a self) -> Option<&'a str>{
        self.validate.value.clone()
    }
}
