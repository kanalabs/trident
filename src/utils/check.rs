use crate::{
    utils::error::HealthError, utils::aptos::requests::check_aptos_response_status, Rpc, Settings,
};

use anyhow::Result;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::time::sleep;

/// Call check and safe_block in a loop
pub async fn health_check(
    rpc_list: Arc<RwLock<Vec<Rpc>>>,
    config: &Arc<RwLock<Settings>>,
) -> Result<(), HealthError> {
    loop {
        let health_check_ttl = config.read().unwrap().health_check_ttl;

        sleep(Duration::from_millis(health_check_ttl)).await;

        check(&rpc_list).await?;
    }
}


impl From<reqwest::Error> for HealthError {
    fn from(error: reqwest::Error) -> Self {
        HealthError::GetSafeBlockError(error.to_string())
    }
}

async fn check(rpc_list: &Arc<RwLock<Vec<Rpc>>>) -> Result<(), HealthError> {
    check_aptos_response_status(rpc_list).await
}
