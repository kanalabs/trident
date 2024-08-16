use std::sync::{Arc, RwLock};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{utils::error::HealthError, utils::rpc::Rpc};

#[derive(Serialize, Deserialize, Debug)]
struct AptosApiResponse {
    chain_id: u32,
    epoch: String,
    ledger_version: String,
    oldest_ledger_version: String,
    ledger_timestamp: String,
    node_role: String,
    oldest_block_height: String,
    block_height: String,
    git_hash: String,
}

pub async fn send_health_request(
    url: String,
    client: Client,
) -> Result<String, crate::utils::error::RpcError> {
    #[cfg(feature = "debug-verbose")]
    println!("Sending request: {}", tx.clone());
    let url = format!("{}/v1", url);
    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(err) => {
            return Err(crate::utils::error::RpcError::InvalidResponse(
                err.to_string(),
            ))
        }
    };
    #[cfg(not(feature = "debug-verbose"))]
    Ok(response.text().await.unwrap())
}

pub fn is_valid_api_response(json_response: &str) -> bool {
    serde_json::from_str::<AptosApiResponse>(json_response).is_ok()
}

pub async fn check_aptos_rpc_status(
    rpc_list: &Arc<RwLock<Vec<Rpc>>>,
    poverty_list: &Arc<RwLock<Vec<Rpc>>>,
) -> Result<(), HealthError> {
    let rpc_clone = rpc_list.read().unwrap().clone();
    let mut status: bool = true;
    let mut to_remove = Vec::new();
    let mut to_add = Vec::new();

    for rpc in &rpc_clone {
        let client = reqwest::Client::new();
        let url = format!("{}/v1", &rpc.url);
        let response = client.get(url).send().await?;
        println!("RPC IN LIST {:?}", &rpc.name);

        if response.status() == 200 {
            println!(
                "APTOS RPC CHECK {:?} : OK! {:?}",
                &rpc.name,
                response.status()
            );
        } else {
            println!("APTOS RPC CHECK {:?} : FAILED!", &rpc.name);
            status = false;
            to_remove.push(rpc.clone());
        }
    }

    let poverty_clone = poverty_list.read().unwrap().clone();
    for rpc in &poverty_clone {
        let client = reqwest::Client::new();
        let url = format!("{}/v1", &rpc.url);
        let response = client.get(url).send().await?;
        println!("RPC IN POVERTY LIST {:?}", &rpc.name);

        if response.status() == 200 {
            println!(
                "RPC BACK ONLINE {:?} : OK! {:?}",
                &rpc.name,
                response.status()
            );
            to_add.push(rpc.clone());
        } else {
            println!("APTOS RPC CHECK {:?} : FAILED!", &rpc.name);
            status = false;
        }
    }

    // Now we acquire the write lock
    let mut rpc_list_guard = rpc_list.write().unwrap();
    let mut poverty_list_guard = poverty_list.write().unwrap();

    for rpc in to_remove.iter() {
        println!("Removing RPC from list {:?} ", &rpc.name);
        rpc_list_guard.retain(|r| r.url != rpc.url);
        poverty_list_guard.push(rpc.clone());
    }

    for rpc in to_add.iter() {
        println!("Adding RPC back to list {:?}", &rpc.name);
        poverty_list_guard.retain(|r| r.url != rpc.url);
        rpc_list_guard.push(rpc.clone());
    }

    if status {
        Ok(())
    } else {
        Err(HealthError::Unresponsive)
    }
}

