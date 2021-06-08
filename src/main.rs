use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::error;
use url::Url;


type BoxResult<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

#[derive(Default)]
struct RequestQueries {
    nonce: String,              // int, ever-increasing u64
    userref: Option<String>,    // int, user reference id
    ordertype: String,          // string, market, limit, stop-loss, take-profit, stop-loss-limit, take-profit-limit, settle-position
    r#type: String,             // string, buy/sell
    volume: Option<String>,     // Order quantity in terms of the base asset
    pair: String,               // Crypto pair
    price: Option<String>,      // Limit price for limit orders, Trigger price for stop-loss, stop-loss-limit, take-profit and take-profit-limit orders
    price2: Option<String>,     // Limit price for stop-loss-limit and take-profit-limit orders
    leverage: Option<String>,   // Amount of leverage desired (default = none)
    oflags: Option<String>,     // Comma delimited list of order flags
    starttm: Option<String>,    // string, Scheduled start time
    expiretm: Option<String>,   // string, Expiration time
    validate: Option<String>    // Validate inputs only. Do not submit order
}

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

    let private_key = "kQH5HW/8p1uGOVjbgWA7FunAmGO8lsSUXNsu3eow76sz84Q18fWxnyRzBHCd3pd5nE9qa99HAZtuZuj6F1huXg==";


    Ok(())

}


impl RequestQueries {
    pub fn new() -> RequestQueries { RequestQueries::default() }

    pub fn set_nonce<'a>(&'a mut self, value: String) -> &'a RequestQueries {
        self.nonce = value;
        self
    }

    pub fn set_userref<'a>(&'a mut self, value: String) -> &'a RequestQueries {
        self.userref = Some(value);
        self
    }

    pub fn set_ordertype<'a>(&'a mut self, value: String) -> &'a RequestQueries {
        self.ordertype = value;
        self
    }

    pub fn set_type<'a>(&'a mut self, value: String) -> &'a RequestQueries {
        self.r#type = value;
        self
    }

    pub fn set_volume<'a>(&'a mut self, value: String) -> &'a RequestQueries {
        self.volume = Some(value);
        self
    }

}
