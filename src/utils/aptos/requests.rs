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

pub async fn check_aptos_response_status(rpc_list: &Arc<RwLock<Vec<Rpc>>>) -> Result<(), HealthError> {
    let rpc_clone = rpc_list.read().unwrap().clone();

    let len = rpc_clone.len();
    let mut status: bool = true;
    for i in 0..len {
        let client = reqwest::Client::new();
        let url = format!("{}/v1", &rpc_clone[i].url);
        let response = client.get(url).send().await?;
        if response.status() == 200 {
            println!("APTOS RPC CHECK {:?} : OK!", &rpc_clone[i].name);
            status = true;
        } else {
            status = false;
        }
    }

    if status == true {
        Ok(())
    } else {
        Err(HealthError::Unresponsive)
    }
}
