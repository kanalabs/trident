
use std::error::Error;

#[derive(Debug)]
#[allow(dead_code)]
pub enum HealthError {
    Unresponsive,
    TimedOut,
    GetSafeBlockError(String),
    //InvalidHexFormat,
    OutOfBounds,
    InvalidResponse(String),
}


#[derive(Debug)]
#[allow(dead_code)]
pub enum RpcError {
    Unresponsive,
    //InvalidHexFormat,
    OutOfBounds,
    InvalidResponse(String),
}

impl std::fmt::Display for HealthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HealthError::Unresponsive => write!(f, "RPC is unresponsive"),
            HealthError::TimedOut => write!(f, "Health check timed out!"),
            HealthError::GetSafeBlockError(reason) => {
                write!(f, "Could not get safe block: {}", reason)
            }
            HealthError::OutOfBounds => {
                write!(
                    f,
                    "Request out of bounds. Most likeley a bad response from the current RPC node."
                )
            }
            HealthError::InvalidResponse(reason) => write!(f, "Invalid RPC response: {}", reason),
        }
    }
}

impl Error for HealthError {}

impl From<RpcError> for HealthError {
    fn from(error: RpcError) -> Self {
        HealthError::GetSafeBlockError(error.to_string())
    }
}


impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RpcError::Unresponsive => write!(f, "RPC is unresponsive"),
            RpcError::OutOfBounds => {
                write!(
                    f,
                    "Request out of bounds. Most likeley a bad response from the current RPC node."
                )
            }
            RpcError::InvalidResponse(reason) => write!(f, "Invalid RPC response: {}", reason),
        }
    }
}

impl From<simd_json::Error> for RpcError {
    fn from(_: simd_json::Error) -> Self {
        RpcError::InvalidResponse("Error while trying to parse JSON".to_string())
    }
}

impl Error for RpcError {}