# Trident 🔱

Trident is a generic light weight load balancer for aptos RPC's , This is heavily inspired by blutgang (A light weight load balancer built for ethereum RPC's) and contains sources from it

Check out [Blutgang](https://github.com/rainshowerLabs/blutgang) !



NOTE : THE LOAD BALANCER IS STILL UNDER DEVELOPMENT....

# Starting the load balancer

```Shell
cargo run --release -- -c config.toml
```

# Using cargo watch 

```Shell
cargo watch -x 'run --release -- -c config.toml'
```

# Run with Docker 

NOTE : By default example.config.json is used as config file , to use custom configurations mount the config file while running docker

```shell
docker build -t trident .
docker run -d --name trident_container -p 3001:3001 -v ./config.toml:/app/config.toml trident

```

# 

# Example config.toml (Some of the features may not work)

```toml

# Config for trident 
[trident]
# Where to bind trident to
address = "0.0.0.0:3001"
# Moving average length for the latency
ma_length = 100
# Sort RPCs by latency on startup. Recommended to leave on.
sort_on_startup = true
# Enable health checking
health_check = true
# Acceptable time to wait for a response in ms
ttl = 60000
# How many times to retry a request before giving up
max_retries = 32
# Time between health checks in ms
health_check_ttl = 15000

[public]
url = "https://api.mainnet.aptoslabs.com"
# The maximum amount of time we can use this rpc in a row.
max_consecutive = 15
# Max amount of queries per second.
max_per_second = 15


[AnkrPublic]
url = "https://rpc.ankr.com/http/aptos"
# The maximum amount of time we can use this rpc in a row.
max_consecutive = 15
# Max amount of queries per second.
max_per_second = 15

```

## License

Trident is licensed under MIT 

