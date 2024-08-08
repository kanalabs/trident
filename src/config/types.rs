use crate::{config::setup::sort_by_latency, log_info, Rpc};
use clap::{ArgMatches, Command};
use jsonwebtoken::DecodingKey;

use std::{
    fmt,
    fmt::Debug,
    fs::{self},
    net::SocketAddr,
    println,
};

use toml::Value;

#[derive(Clone)]
pub struct AdminSettings {
    pub enabled: bool,
    pub address: SocketAddr,
    pub readonly: bool,
    pub jwt: bool,
    pub key: DecodingKey,
}

impl Default for AdminSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            address: "127.0.0.1:3001".parse::<SocketAddr>().unwrap(),
            readonly: false,
            jwt: false,
            key: DecodingKey::from_secret(b""),
        }
    }
}

impl Debug for AdminSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AdminSettings {{")?;
        write!(f, " enabled: {:?}", self.enabled)?;
        write!(f, ", address: {:?}", self.address)?;
        write!(f, ", readonly: {:?}", self.readonly)?;
        write!(f, ", jwt: HIDDEN",)?;
        write!(f, " }}")
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub rpc_list: Vec<Rpc>,
    pub poverty_list: Vec<Rpc>,
    pub address: SocketAddr,
    pub health_check: bool,
    pub ttl: u128,
    pub max_retries: u32,
    pub health_check_ttl: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rpc_list: Vec::new(),
            poverty_list: Vec::new(),
            address: "127.0.0.1:3000".parse::<SocketAddr>().unwrap(),
            health_check: false,
            ttl: 1000,
            max_retries: 32,
            health_check_ttl: 1000,
        }
    }
}

impl Settings {
    pub async fn new(matches: Command) -> Settings {
        let matches = matches.get_matches();

        // Try to open the file at the path specified in the args
        let path = matches.get_one::<String>("config").unwrap();
        let file: Option<String> = match fs::read_to_string(path) {
            Ok(file) => Some(file),
            Err(_) => panic!("\x1b[31mErr:\x1b[0m Error opening config file at {}", path),
        };

        if let Some(file) = file {
            log_info!("Using config file at {}", path);
            return Settings::create_from_file(file).await;
        }

        log_info!("Using command line arguments for settings...");
        Settings::create_from_matches(matches)
    }

    async fn create_from_file(conf_file: String) -> Settings {
        let parsed_toml = conf_file.parse::<Value>().expect("Error parsing TOML");

        let table_names: Vec<&String> = parsed_toml.as_table().unwrap().keys().collect::<Vec<_>>();

        // Parse the `trident` table
        let trident_table = parsed_toml
            .get("trident")
            .expect("\x1b[31mErr:\x1b[0m Missing trident table!")
            .as_table()
            .expect("\x1b[31mErr:\x1b[0m Could not parse trident table!");
        let address = trident_table
            .get("address")
            .expect("\x1b[31mErr:\x1b[0m Missing address!")
            .as_str()
            .expect("\x1b[31mErr:\x1b[0m Could not parse address as str!");
        let sort_on_startup = trident_table
            .get("sort_on_startup")
            .expect("\x1b[31mErr:\x1b[0m Missing sort_on_startup toggle!")
            .as_bool()
            .expect("\x1b[31mErr:\x1b[0m Could not parse sort_on_startup as bool!");

        // Build the SocketAddr
        let port = 3000;
        // Replace `localhost` if it exists
        let address = address.replace("localhost", "127.0.0.1");
        // If the address contains `:` dont concatanate the port and just pass the address
        let address = if address.contains(':') {
            address.to_string()
        } else {
            format!("{}:{}", address, port)
        };
        let address = address
            .parse::<SocketAddr>()
            .expect("\x1b[31mErr:\x1b[0m Could not address to SocketAddr!");

        let ma_length = trident_table
            .get("ma_length")
            .expect("\x1b[31mErr:\x1b[0m Missing ma_length!")
            .as_integer()
            .expect("\x1b[31mErr:\x1b[0m Could not parse ma_length as int!")
            as f64;

        let health_check = trident_table
            .get("health_check")
            .expect("\x1b[31mErr:\x1b[0m Missing health_check toggle!")
            .as_bool()
            .expect("\x1b[31mErr:\x1b[0m Could not parse health_check as bool!");
        let ttl = trident_table
            .get("ttl")
            .expect("\x1b[31mErr:\x1b[0m Missing ttl!")
            .as_integer()
            .expect("\x1b[31mErr:\x1b[0m Could not parse ttl as int!") as u128;

        let max_retries = trident_table
            .get("max_retries")
            .expect("\x1b[31mErr:\x1b[0m Missing max_retries!")
            .as_integer()
            .expect("\x1b[31mErr:\x1b[0m Could not parse max_retries as int!")
            as u32;

        let health_check_ttl = if health_check {
            trident_table
                .get("health_check_ttl")
                .expect("\x1b[31mErr:\x1b[0m Missing health_check_ttl!")
                .as_integer()
                .expect("\x1b[31mErr:\x1b[0m Could not parse health_check_ttl as int!")
                as u64
        } else {
            u64::MAX
        };

        let mut rpc_list: Vec<Rpc> = Vec::new();
        for table_name in table_names {
            if table_name != "trident" && table_name != "sled" && table_name != "admin" {
                let rpc_table = parsed_toml.get(table_name).unwrap().as_table().unwrap();

                let max_consecutive = rpc_table
                    .get("max_consecutive")
                    .expect("\x1b[31mErr:\x1b[0m Missing max_consecutive from an RPC!")
                    .as_integer()
                    .expect("\x1b[31mErr:\x1b[0m Could not parse max_consecutive as int!")
                    as u32;

                let mut delta = rpc_table
                    .get("max_per_second")
                    .expect("\x1b[31mErr:\x1b[0m Missing max_per_second from an RPC!")
                    .as_integer()
                    .expect("\x1b[31mErr:\x1b[0m Could not parse max_per_second as int!")
                    as u64;

                // If the delta time isnt 0, we need to get how many microsecond need to pass
                // before we can send a new request
                if delta != 0 {
                    delta = 1_000_000 / delta;
                }

                let url = rpc_table
                    .get("url")
                    .expect("\x1b[31mErr:\x1b[0m Missing URL from RPC!")
                    .as_str()
                    .expect("\x1b[31mErr:\x1b[0m Could not parse URL from a RPC as str!")
                    .to_string();

                // ws_url is an Option<>
                //
                // If we cant read it it should be `None`
                let ws_url = match rpc_table.get("ws_url") {
                    Some(ws_url) => Some(
                        ws_url
                            .as_str()
                            .expect("\x1b[31mErr:\x1b[0m Could not parse ws_url as str!")
                            .to_string(),
                    ),
                    None => {
                        None
                    }
                };

                let rpc = Rpc::new(url, ws_url, max_consecutive, delta.into(), ma_length);
                rpc_list.push(rpc);
            }
        }

        let mut poverty_list = Vec::new();
        if sort_on_startup {
            (rpc_list, poverty_list) =
                match sort_by_latency(rpc_list, poverty_list, ma_length).await {
                    Ok(rax) => rax,
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                };
        }

        Settings {
            rpc_list,
            poverty_list,
            address,
            health_check,
            ttl,
            max_retries,
            health_check_ttl,
        }
    }

    fn create_from_matches(matches: ArgMatches) -> Settings {
        // Build the rpc_list
        let rpc_list: String = matches
            .get_one::<String>("rpc_list")
            .expect("Invalid rpc_list")
            .to_string();

        let ma_length = matches
            .get_one::<String>("ma_length")
            .expect("Invalid ma_length");
        let ma_length = ma_length.parse::<f64>().expect("Invalid ma_length");

        let mut delta = matches
            .get_one::<u64>("max_per_second")
            .expect("Invalid max_per_second")
            .to_owned();

        if delta != 0 {
            delta = 1_000_000 / delta;
        }

        // Turn the rpc_list into a csv vec
        let rpc_list: Vec<&str> = rpc_list.split(',').collect();
        let rpc_list: Vec<String> = rpc_list.iter().map(|rpc| rpc.to_string()).collect();
        // Make a list of Rpc structs
        let rpc_list: Vec<Rpc> = rpc_list
            .iter()
            .map(|rpc| Rpc::new(rpc.to_string(), None, 6, delta.into(), ma_length))
            .collect();

        // Build the SocketAddr
        let address = matches
            .get_one::<String>("address")
            .expect("Invalid address");
        let port = matches.get_one::<String>("port").expect("Invalid port");
        // If the address contains `:` dont concatanate the port and just pass the address
        let address = if address.contains(':') {
            address.to_string()
        } else {
            format!("{}:{}", address, port)
        };

        let address = address
            .parse::<SocketAddr>()
            .expect("Invalid address or port!");



        let health_check = matches.get_occurrences::<String>("health_check").is_some();

        let ttl = matches
            .get_one::<String>("ttl")
            .expect("Invalid ttl")
            .parse::<u128>()
            .expect("Invalid ttl");

        let max_retries = matches
            .get_one::<String>("max_retries")
            .expect("Invalid max_retries")
            .parse::<u32>()
            .expect("Invalid max_retries");

        let health_check_ttl = matches
            .get_one::<String>("health_check_ttl")
            .expect("Invalid health_check_ttl")
            .parse::<u64>()
            .expect("Invalid health_check_ttl");


        Settings {
            rpc_list,
            poverty_list: Vec::new(),
            address,
            health_check,
            ttl,
            max_retries,
            health_check_ttl,
        }
    }
}
