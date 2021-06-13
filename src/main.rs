use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::error;
use url::Url;
use sha2::{Sha256, Sha512, Digest};
use data_encoding::BASE64;
use std::error::Error;
use hmac::{Hmac, Mac, NewMac};



type BoxResult<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;


#[derive(Default, Debug, Clone)]
struct Query<'a> {
    name: &'a str,
    required: bool,
    value: Option<&'a str>
}

#[derive(Default, Debug)]
struct RequestQueries<'a> {
    nonce: Query<'a>,      // int, ever-increasing u64
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

type HmacSha512 = Hmac<Sha512>;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> BoxResult<()> {
    let opts = App::new("mongodb-stream-rs")
        .version(crate_version!())
        .author("Daniel F. <dan@findelabs.com>")
        .about("Stream MongoDB to MongoDB")
        .arg(
            Arg::with_name("private_key")
                .long("private_key")
                .required(true)
                .value_name("PRIVATE_KEY")
                .env("PRIVATE_KEY")
                .help("Private Key")
                .takes_value(true),
        )
        .get_matches();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"message\": \"{}\"}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();


    let base = "https://api.kraken.com";
    let path = "/0/private/AddOrder";

    let mut kraken_url = Url::parse(base)?;
    kraken_url.set_path(path);

    let mut queries = RequestQueries::new();
    queries.set_nonce("1616492376594");
    queries.set_type("buy");
    queries.set_ordertype("limit");
    queries.set_pair("XBTUSD");
    queries.set_price("37500");
    queries.set_volume("1.25");

    let private_key = &opts.value_of("private_key").unwrap();

    signature(queries, private_key);

    Ok(())

}

fn signature(mut queries: RequestQueries, private_key: &str) -> Result<String,Box<dyn Error + Send + Sync>> {
    let path = "/0/private/AddOrder";
    let postdata = queries.queries().unwrap();
    let encoded = queries.nonce().unwrap().to_string() + &postdata;

    let mut hasher = Sha256::new();
    hasher.update(encoded);
    let result = hasher.finalize();

    let mut message = path.as_bytes().to_vec();
    for elem in result {
        message.push(elem);
    }

    let hmac_key = BASE64.decode(private_key.as_bytes())?;
    let mut mac = Hmac::<Sha512>::new_from_slice(&hmac_key).expect("here");
    mac.update(&message);
    let mac_result = mac.finalize();

    let b64 = BASE64.encode(&mac_result.into_bytes());

    println!("{}", b64);

    Ok(b64)
}


impl<'b> RequestQueries<'b> {
    pub fn new() -> RequestQueries<'static> { 
        RequestQueries {
            nonce: Query {name: "nonce", required: true, value: None},
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

    pub fn queries(&self) -> Option<String> {
        let queries: [&str; 13] = [
            "nonce",
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
            "nonce" => Some(self.nonce.clone()),
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

    pub fn set_nonce<'a>(&'a mut self, value: &'b str) -> &'a RequestQueries {
        self.nonce.value = Some(value);
        self
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

    pub fn nonce<'a>(&'a mut self) -> Option<&'a str> {
        self.nonce.value.as_deref()
    }

    pub fn userref<'a>(&'a mut self) -> Option<&'a str> {
        self.userref.value.as_deref()
    }

    pub fn ordertype<'a>(&'a mut self) -> Option<&'a str> {
        self.ordertype.value.as_deref()
    }

    pub fn r#type<'a>(&'a mut self) -> Option<&'a str> {
        self.r#type.value.as_deref()
    }

    pub fn volume<'a>(&'a mut self) -> Option<&'a str>{
        self.volume.value.as_deref()
    }

    pub fn pair<'a>(&'a mut self) -> Option<&'a str> {
        self.pair.value.as_deref()
    }

    pub fn price<'a>(&'a mut self) -> Option<&'a str>{
        self.price.value.as_deref()
    }

    pub fn price2<'a>(&'a mut self) -> Option<&'a str>{
        self.price2.value.as_deref()
    }

    pub fn leverage<'a>(&'a mut self) -> Option<&'a str>{
        self.leverage.value.as_deref()
    }

    pub fn oflags<'a>(&'a mut self) -> Option<&'a str>{
        self.oflags.value.as_deref()
    }

    pub fn starttm<'a>(&'a mut self) -> Option<&'a str>{
        self.starttm.value.as_deref()
    }

    pub fn expiretm<'a>(&'a mut self) -> Option<&'a str>{
        self.expiretm.value.as_deref()
    }

    pub fn validate<'a>(&'a mut self) -> Option<&'a str>{
        self.validate.value.as_deref()
    }
}
