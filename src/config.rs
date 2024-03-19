use std::net::IpAddr;

use serde::Deserialize;

fn default_ip() -> IpAddr {
    IpAddr::from([0, 0, 0, 0])
}

fn default_port() -> u16 {
    50051
}

fn default_flightmngr_url() -> String {
    String::from("grpc://flightmngr")
}

fn default_rabbitmq_url() -> String {
    String::from("localhost")
}

fn default_rabbitmq_port() -> u16 {
    5672
}

fn default_rabbitmq_user() -> String {
    String::from("guest")
}

fn default_rabbitmq_password() -> String {
    String::from("hard0ne")
}

#[derive(Deserialize, Debug)]
pub struct Options {
    pub database_url: String,
    #[serde(default = "default_ip")]
    pub ip: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_flightmngr_url")]
    pub flightmngr_url: String,
    #[serde(default = "default_rabbitmq_url")]
    pub rabbitmq_url: String,
    #[serde(default = "default_rabbitmq_port")]
    pub rabbitmq_port: u16,
    #[serde(default = "default_rabbitmq_user")]
    pub rabbitmq_user: String,
    #[serde(default = "default_rabbitmq_password")]
    pub rabbitmq_password: String,
}
