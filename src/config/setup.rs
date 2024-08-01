use crate::{config::error::ConfigError, log_err, Rpc};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug)]
enum StartingLatencyResp {
    Ok(Rpc),
    Error(Rpc, ConfigError),
}

/// Get the average latency for a RPC
async fn set_starting_latency(
    mut rpc: Rpc,
    ma_length: f64,
    tx: mpsc::Sender<StartingLatencyResp>,
) -> Result<(), ConfigError> {
    let mut latencies = Vec::new();

    for _ in 0..ma_length as u32 {
        let start = Instant::now();
        match rpc.syncing().await {
            Ok(true) => {}
            Ok(false) => {
                tx.send(StartingLatencyResp::Error(rpc, ConfigError::Syncing()))
                    .await?;
                return Err(ConfigError::RpcError("Node syncing to head".to_string()));
            }
            Err(e) => {
                tx.send(StartingLatencyResp::Error(rpc, e.into())).await?;
                return Err(ConfigError::RpcError(
                    "Error awaiting sync status!".to_string(),
                ));
            }
        };
        let end = Instant::now();
        let latency = end.duration_since(start).as_nanos() as f64;
        latencies.push(latency);
    }

    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    rpc.update_latency(avg_latency);

    println!("{}: {}ns", rpc.name, rpc.status.latency);

    tx.send(StartingLatencyResp::Ok(rpc)).await?;

    Ok(())
}

pub async fn sort_by_latency(
    mut rpc_list: Vec<Rpc>,
    mut poverty_list: Vec<Rpc>,
    ma_length: f64,
) -> Result<(Vec<Rpc>, Vec<Rpc>), ConfigError> {
    // Return empty vec if we dont supply any RPCs
    if rpc_list.is_empty() {
        log_err!("No RPCs supplied!");
        return Ok((Vec::new(), Vec::new()));
    }

    let (tx, mut rx) = mpsc::channel(rpc_list.len());

    // Iterate over each RPC
    for rpc in rpc_list.drain(..) {
        let tx = tx.clone();
        // Spawn a new asynchronous task for each RPC
        tokio::spawn(set_starting_latency(rpc, ma_length, tx));
    }

    let mut sorted_rpc_list = Vec::new();

    // Drop tx so we don't try to receive nothing
    drop(tx);

    // Collect results from tasks
    while let Some(rpc) = rx.recv().await {
        let rpc = match rpc {
            StartingLatencyResp::Ok(rax) => rax,
            StartingLatencyResp::Error(mut rax, e) => {
                log_err!("Adding to poverty list: {}", e);
                rax.status.is_erroring = true;
                poverty_list.push(rax);
                continue;
            }
        };
        sorted_rpc_list.push(rpc);
    }

    // Sort the RPCs by latency
    sorted_rpc_list.sort_by(|a, b| a.status.latency.partial_cmp(&b.status.latency).unwrap());

    Ok((sorted_rpc_list, poverty_list))
}

