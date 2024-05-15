use std::net::{IpAddr, Ipv4Addr, SocketAddr, AddrParseError};
use std::str::FromStr;
use std::{fs::File, io::BufReader};

use axum::{http::{StatusCode, HeaderMap}, response::IntoResponse, routing::get, Json, Router};
use axum::extract::ConnectInfo;
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
    tracing::info!("Config is {:?}", &config);

    let app = Router::new()
        .route("/", get(root))
        .route("/json", get(get_ip_json));

    let listener = tokio::net::TcpListener::bind(config.bind_address.0)
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

fn get_ip(_headers: &HeaderMap, addr: SocketAddr) -> IpAddr {
    addr.ip()
}

async fn root(headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    tracing::trace!("Get request: /");
    get_ip(&headers, addr).to_string()
}

#[derive(Serialize)]
struct Ip {
    ip: IpAddr,
}

async fn get_ip_json(headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    let ip = get_ip(&headers, addr);
    let ip = Ip { ip };
    (StatusCode::OK, Json(ip))
}
