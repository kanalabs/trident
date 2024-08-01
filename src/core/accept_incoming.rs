use crate::{
    core::{processing::update_rpc_latency, algo::pick},
    log_info, log_wrn, no_rpc_available,
    utils::rpc::Rpc,
    timed_out, Settings,
};
use futures::executor::block_on;
use http::request::Parts;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request};
use std::{
    convert::Infallible,
    println,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::sync::watch;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub rpc_list_rwlock: Arc<RwLock<Vec<Rpc>>>,
    pub channels: RequestChannels,
    // pub head_cache: Arc<RwLock<BTreeMap<u64, Vec<String>>>>,
    // pub sub_data: Arc<SubscriptionData>,
    // pub cache: Db,
    pub config: Arc<RwLock<Settings>>,
}

impl ConnectionParams {
    pub fn new(
        rpc_list_rwlock: &Arc<RwLock<Vec<Rpc>>>,
        channels: RequestChannels,
        config: &Arc<RwLock<Settings>>,
    ) -> Self {
        ConnectionParams {
            rpc_list_rwlock: rpc_list_rwlock.clone(),
            channels,
            config: config.clone(),
        }
    }
}

struct RequestParams {
    ttl: u128,
    max_retries: u32,
}

#[derive(Debug)]
pub struct RequestChannels {
    pub finalized_rx: Arc<watch::Receiver<u64>>,
}

impl RequestChannels {
    pub fn new(finalized_rx: Arc<watch::Receiver<u64>>) -> Self {
        Self { finalized_rx }
    }
}

impl Clone for RequestChannels {
    fn clone(&self) -> Self {
        Self {
            finalized_rx: Arc::clone(&self.finalized_rx),
        }
    }
}

/// Macros for accepting requests
#[macro_export]
macro_rules! accept {
    (
        $io:expr,
        $connection_params:expr
    ) => {
        // Bind the incoming connection to our service
        if let Err(err) = http1::Builder::new()
            // `service_fn` converts our function in a `Service`
            .serve_connection(
                $io,
                service_fn(|req| {
                    let response = accept_request(req, $connection_params);
                    response
                }),
            )
            .with_upgrades()
            .await
        {
            log_err!("Error serving connection: {:?}", err);
        }
    };
}

/// Macro for getting responses from either the cache or RPC nodes
macro_rules! get_response {
    (
        $cache:expr,
        $rpc_position:expr,
        $rpc_list_rwlock:expr,
        $finalized_rx:expr,
        $named_numbers:expr,
        $head_cache:expr,
        $ttl:expr,
        $max_retries:expr,
        $parts:expr,
        $bytes:expr
    ) => {
        fetch_from_rpc!(
            $rpc_list_rwlock,
            $rpc_position,
            $cache,
            $finalized_rx,
            $named_numbers,
            $head_cache,
            $ttl,
            $max_retries,
            $parts,
            $bytes
        )
    };
}

macro_rules! fetch_from_rpc {
    (
        $rpc_list_rwlock:expr,
        $rpc_position:expr,
        $cache:expr,
        $finalized_rx:expr,
        $named_numbers:expr,
        $head_cache:expr,
        $ttl:expr,
        $max_retries:expr,
        $parts:expr,
        $bytes:expr
    ) => {{

        // Loop until we get a response
        let  rx;
        let status;
        let mut retries = 0;
        let mut rpc_name;
        loop {
            // Get the next Rpc in line.
            let mut rpc;
            {
                let mut rpc_list = $rpc_list_rwlock.write().unwrap();
                (rpc, $rpc_position) = pick(&mut rpc_list);
            }
            rpc_name = rpc.name.clone();
            log_info!("Forwarding to: {}", rpc_name);
            // Check if we have any RPCs in the list, if not return error
            if $rpc_position == None {
                return (no_rpc_available!(), None);
            }

            // Send the request. And return a timeout if it takes too long
            //
            // Check if it contains any errors or if its `latest` and insert it if it isn't
            match timeout(
                Duration::from_millis($ttl.try_into().unwrap()),
                rpc.send_request($parts,$bytes)
            )
            .await
            {
             Ok(rxa) => {
                match rxa {
                    Ok(value) => {
                        (rx,status) = value;
                        break;
                    },
                    Err(e) => {
                        log_wrn!("\x1b[93mWrn:\x1b[0m An RPC request in {} has failed : {}",rpc.name,e);
                        log_wrn!("\x1b[93mWrn:\x1b[0m Picking new RPC and retrying.");
                        rpc.update_latency($ttl as f64);
                        retries += 1;
                    }
                }
                },
                Err(_) => {
                    log_wrn!("\x1b[93mWrn:\x1b[0m An RPC request has timed out, picking new RPC and retrying.");
                    rpc.update_latency($ttl as f64);
                    retries += 1;
                },
            };

            if retries == $max_retries {
                return (timed_out!(), $rpc_position,);
            }
        }


        (rx,status,rpc_name)
    }};
}

/// Pick RPC and send request to it. In case the result is cached,
/// read and return from the cache.
async fn forward_body(
    rpc_list_rwlock: &Arc<RwLock<Vec<Rpc>>>,
    params: RequestParams,
    parts: Parts,
    bytes: Bytes,
) -> (
    Result<hyper::Response<Full<Bytes>>, Infallible>,
    Option<usize>,
) {
    // RPC used to get the response, we use it to update the latency for it later.
    let mut rpc_position;

    // Get the response from either the DB or from a RPC. If it timeouts, retry.
    let (rax, status, rpc_name) = get_response!(
        cache,
        rpc_position,
        rpc_list_rwlock,
        finalized_rx.clone(),
        named_numbers.clone(),
        head_cache.clone(),
        params.ttl,
        params.max_retries,
        parts.clone(),
        bytes.clone()
    );

    let res;
    // Convert rx to bytes and but it in a Buf
    let body = hyper::body::Bytes::from(rax);
    // Put it in a http_body_util::Full
    let body = Full::new(body);

    if status == 200 {
        // Build the response
        res = hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .header("rpc-used", rpc_name)
            .body(body)
            .unwrap();
    } else {
        res = hyper::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .header("rpc-used", rpc_name)
            .body(body)
            .unwrap();
    }

    (Ok(res), rpc_position)
}

pub async fn accept_request(
    tx: Request<hyper::body::Incoming>,
    connection_params: ConnectionParams,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    // Send request and measure time
    let response: Result<hyper::Response<Full<Bytes>>, Infallible>;
    let rpc_position: Option<usize>;

    // RequestParams from config
    let params = {
        let config_guard = connection_params.config.read().unwrap();
        RequestParams {
            ttl: config_guard.ttl,
            max_retries: config_guard.max_retries,
        }
    };

    let time = Instant::now();

    // get body and parts from incoming request
    let (parts, body) = tx.into_parts();
    // let body_bytes = body.collect();
    let body_bytes = block_on(async {
        body.collect()
            .await
            .map(|collected| collected.to_bytes())
            .unwrap_or_else(|_| Bytes::new())
    });

    (response, rpc_position) = forward_body(
        &connection_params.rpc_list_rwlock,
        params,
        parts.clone(),
        body_bytes.clone(),
    )
    .await;
    let time = time.elapsed();
    log_info!("Request time: {:?}", time);

    // `rpc_position` is an Option<> that either contains the index of the RPC
    // we forwarded our request to, or is None if the result was cached.
    //
    // Here, we update the latency of the RPC that was used to process the request
    // if `rpc_position` is Some.
    if let Some(rpc_position) = rpc_position {
        update_rpc_latency(&connection_params.rpc_list_rwlock, rpc_position, time);
    }

    response
}
