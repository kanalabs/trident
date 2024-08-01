use crate::config::system::VERSION_STR;

use clap::{Arg, Command};

pub fn create_match() -> clap::Command {
    Command::new("trident")
        .version(VERSION_STR)
        .author("mohan <mohan@kanalabs.io> and contributors")
        .about("trident load balancer ")
        .arg(
            Arg::new("rpc_list")
                .long("rpc_list")
                .short('r')
                .num_args(1..)
                .default_value("")
                .conflicts_with("config")
                .help("CSV list of rpcs"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .num_args(1..)
                .default_value("config.toml")
                .conflicts_with("rpc_list")
                .help("TOML config file for trident"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .num_args(1..)
                .default_value("3000")
                .help("Port to listen to"),
        )
        .arg(
            Arg::new("address")
                .long("address")
                .short('a')
                .num_args(1..)
                .default_value("127.0.0.1")
                .help("Address to bind to"),
        )
        .arg(
            Arg::new("ma_length")
                .long("ma_length")
                .num_args(1..)
                .default_value("15")
                .help("Latency moving average length"),
        )
        .arg(
            Arg::new("")
                .long("health_check")
                .num_args(0..)
                .help("Enable health checking"),
        )
        .arg(
            Arg::new("ttl")
                .long("ttl")
                .num_args(1..)
                .default_value("300")
                .help("Time for the RPC to respond before we remove it from the active queue"),
        )
        .arg(
            Arg::new("max_retries")
                .long("max_retries")
                .num_args(1..)
                .default_value("32")
                .help("Maximum amount of retries before we drop the current request."),
        )
        .arg(
            Arg::new("health_check_ttl")
                .long("health_check_ttl")
                .num_args(1..)
                .default_value("2000")
                .help("How often to perform the health check"),
        )
}
