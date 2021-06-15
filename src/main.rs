use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::error;
use url::Url;
use queries::RequestQueries;

mod queries;
mod kraken;

type BoxResult<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

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
    queries.set_type("buy");
    queries.set_ordertype("limit");
    queries.set_pair("XBTUSD");
    queries.set_price("37500");
    queries.set_volume("1.25");

    let private_key = &opts.value_of("private_key").unwrap();

    let sig = queries.signature(&path, private_key)?;
    println!("{}", sig);

    Ok(())

}
