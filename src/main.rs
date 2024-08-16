mod config;
mod core;
mod utils;

use crate::{
    config::{cli_args::create_match, types::Settings},
    core::accept_incoming::{accept_request, ConnectionParams, RequestChannels},
    utils::check::health_check,
    utils::rpc::Rpc,
};

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::{net::TcpListener, sync::watch};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util_blutgang::rt::TokioIo;

/// `jemalloc` offers faster mallocs when dealing with lots of threads which is what we're doing
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get all the cli args and set them
    let config = Arc::new(RwLock::new(Settings::new(create_match()).await));

    // Copy the configuration values we need
    let (addr, do_health_check) = {
        let config_guard = config.read().unwrap();
        (config_guard.address, config_guard.health_check)
    };

    let rpc_poverty_list: Arc<RwLock<Vec<Rpc>>> =
        Arc::new(RwLock::new(config.read().unwrap().poverty_list.clone()));

    // Make the list a rwlock
    let rpc_list_rwlock = Arc::new(RwLock::new(config.read().unwrap().rpc_list.clone()));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;
    log_info!("Bound to: {}", addr);

    let (_blocknum_tx, _blocknum_rx) = watch::channel(0);
    let (_finalized_tx, finalized_rx) = watch::channel(0);

    let finalized_rx_arc = Arc::new(finalized_rx.clone());

    if do_health_check {
        let rpc_list_health = Arc::clone(&rpc_list_rwlock);
        let poverty_list_health = Arc::clone(&rpc_poverty_list);
        let health_check_ttl = config.read().unwrap().health_check_ttl;

        tokio::task::spawn(async move {
            loop {
                let _ = health_check(
                    Arc::clone(&rpc_list_health),
                    Arc::clone(&poverty_list_health),
                )
                .await;
                tokio::time::sleep(Duration::from_millis(health_check_ttl)).await;
            }
        });
    }

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, socketaddr) = listener.accept().await?;
        // log_info!("Connection from: {}", socketaddr);
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let channels = RequestChannels::new(finalized_rx_arc.clone());

        let connection_params = ConnectionParams::new(&rpc_list_rwlock, channels, &config);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            accept!(io, connection_params.clone());
        });
    }
}
