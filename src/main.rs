use std::net::{IpAddr, Ipv4Addr, SocketAddr, AddrParseError};
use std::str::FromStr;
use std::{fs::File, io::BufReader};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use serde::Deserialize;
use clap_serde_derive::{
    clap::{self, Parser},
    ClapSerde,
};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Config file
    #[arg(short, long = "config", default_value = "config.yml", value_name = "file")]
    config_path: std::path::PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    config: <Config as ClapSerde>::Opt,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct BindAddress(SocketAddr);

impl Default for BindAddress {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
    }
}

impl FromStr for BindAddress {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(SocketAddr::from_str(s)?))
    }
}

#[derive(ClapSerde, Debug)]
struct Config {
    /// Bind address [default: 127.0.0.1:8080]
    #[arg(short, long, value_name = "SocketAddr")]
    bind_address: BindAddress,
}

fn get_config() -> Config {
    // Parse whole args with clap
    let mut args = Args::parse();

    // Get config file
    if let Ok(f) = File::open(&args.config_path) {
        // Parse config with serde
        match serde_yml::from_reader::<_, <Config as ClapSerde>::Opt>(BufReader::new(f)) {
            // merge config already parsed from clap
            Ok(config) => Config::from(config).merge(&mut args.config),
            Err(err) => panic!("Error in configuration file:\n{}", err),
        }
    } else {
        // If there is not config file return only config parsed from clap
        Config::from(&mut args.config)
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = get_config();
    println!("Config is {:?}", &config);

    let app = Router::new()
        .route("/", get(root))
        .route("/json", get(get_ip_json));

    let listener = tokio::net::TcpListener::bind(config.bind_address.0)
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn get_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

// basic handler that responds with a static string
async fn root() -> String {
    tracing::trace!("Get request: /");
    get_ip().to_string()
}

#[derive(Serialize)]
struct Ip {
    ip: IpAddr,
}

async fn get_ip_json() -> impl IntoResponse {
    let ip = Ip { ip: get_ip() };
    (StatusCode::CREATED, Json(ip))
}
