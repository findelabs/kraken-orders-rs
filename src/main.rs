use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::error;
//use queries::RequestQueries;
use serde_json::{to_value, Value};
use serde_json::map::Map;
use crate::kraken::KrakenClient;

//mod queries;
mod kraken;
mod errors;

type BoxResult<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> BoxResult<()> {
    let opts = App::new("mongodb-stream-rs")
        .version(crate_version!())
        .author("Daniel F. <dan@findelabs.com>")
        .about("Stream MongoDB to MongoDB")
        .arg(
            Arg::with_name("api_key")
                .long("api_key")
                .required(true)
                .value_name("API_KEY")
                .env("API_KEY")
                .help("API Key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("api_secret")
                .long("api_secret")
                .required(true)
                .value_name("API_SECRET")
                .env("API_SECRET")
                .help("API Secret Key")
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


//    let mut kraken_url = Url::parse(base)?;
//    kraken_url.set_path(path);
//
//    let mut queries = RequestQueries::new();
//    queries.set_type("buy");
//    queries.set_ordertype("limit");
//    queries.set_pair("XBTUSD");
//    queries.set_price("37500");
//    queries.set_volume("1.25");

    let api_secret = &opts.value_of("api_secret").unwrap();
    let api_key = &opts.value_of("api_key").unwrap();

    let client = KrakenClient::new(api_key, api_secret);

    let mut payload = Map::new();
    payload.insert("type".to_owned(), Value::String("buy".to_owned()));
    payload.insert("ordertype".to_owned(), Value::String("limit".to_owned()));
    payload.insert("pair".to_owned(), Value::String("XBTUSD".to_owned()));
    payload.insert("price".to_owned(), Value::String("37500".to_owned()));
    payload.insert("volume".to_owned(), Value::String("1.25".to_owned()));
    
//    let result = client.add_order(to_value(payload)?).await?;
    let result = client.balance().await?;
    println!("{}", result);

    Ok(())

}
