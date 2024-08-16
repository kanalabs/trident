use crate::{utils::aptos::requests::check_aptos_rpc_status, utils::error::HealthError, Rpc};

use anyhow::Result;

use std::sync::{Arc, RwLock};

/// Call check and safe_block in a loop
pub async fn health_check(
    rpc_list: Arc<RwLock<Vec<Rpc>>>,
    poverty_list: Arc<RwLock<Vec<Rpc>>>,
) -> Result<(), HealthError> {
    check(&rpc_list, &poverty_list).await?;
    Ok(())
}

impl From<reqwest::Error> for HealthError {
    fn from(error: reqwest::Error) -> Self {
        HealthError::GetSafeBlockError(error.to_string())
    }
}

async fn check(
    rpc_list: &Arc<RwLock<Vec<Rpc>>>,
    poverty_list: &Arc<RwLock<Vec<Rpc>>>,
) -> Result<(), HealthError> {
    check_aptos_rpc_status(rpc_list, poverty_list).await
}
