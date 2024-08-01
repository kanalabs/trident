use crate::Rpc;

use std::{
    collections::BTreeMap,
    println,
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::sync::watch;

use sled::Db;

#[derive(Clone)]
pub struct CacheArgs {
    pub finalized_rx: watch::Receiver<u64>,
    pub cache: Db,
    pub head_cache: Arc<RwLock<BTreeMap<u64, Vec<String>>>>,
}

impl CacheArgs {
    #[allow(dead_code)]
    pub fn default() -> Self {
        CacheArgs {
            finalized_rx: watch::channel(0).1,
            cache: (sled::Config::default().open().unwrap()),
            head_cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}



/// Updates the latency of an RPC node given an rpc list, its position, and the time it took for
/// a request to complete.
pub fn update_rpc_latency(rpc_list: &Arc<RwLock<Vec<Rpc>>>, rpc_position: usize, time: Duration) {
    let mut rpc_list_guard = rpc_list.write().unwrap_or_else(|e| {
        // Handle the case where the RwLock is poisoned
        e.into_inner()
    });

    // Handle weird edge cases ¯\_(ツ)_/¯
    if !rpc_list_guard.is_empty() {
        let index = if rpc_position >= rpc_list_guard.len() {
            rpc_list_guard.len() - 1
        } else {
            rpc_position
        };
        rpc_list_guard[index].update_latency(time.as_nanos() as f64);
        rpc_list_guard[index].last_used = time.as_micros();
        println!("LA {}", rpc_list_guard[index].status.latency);
    }
}
