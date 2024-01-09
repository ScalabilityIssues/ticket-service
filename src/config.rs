use std::net::IpAddr;

use serde::Deserialize;

fn default_ip() -> IpAddr {
    IpAddr::from([0, 0, 0, 0])
}

fn default_port() -> u16 {
    50051
}

#[derive(Deserialize, Debug)]
pub struct Options {
    pub database_url: String,
    #[serde(default = "default_ip")]
    pub ip: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}