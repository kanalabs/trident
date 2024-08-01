use crate::utils::aptos::requests::is_valid_api_response;
use crate::utils::aptos::requests::send_health_request;
use reqwest::Client;
use url::Url;

use http::{HeaderMap, Request};

use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::http::request::Parts;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;

// All as floats so we have an easier time getting averages, stats and terminology copied from flood.
#[derive(Debug, Clone, Default)]
pub struct Status {
    // Set this to true in case the RPC becomes unavailable
    // Also set the last time it was called, so we can check again later
    pub is_erroring: bool,
    pub last_error: u64,

    // The latency is a moving average of the last n calls
    pub latency: f64,
    pub latency_data: Vec<f64>,
    ma_length: f64,
    // ???
    // pub throughput: f64,
}


#[derive(Debug, Clone)]
pub struct Rpc {
    pub name: String,           // sanitized name for appearing in logs
    pub url: String,            // url of the rpc we're forwarding requests to.
    client: Client,             // Reqwest client
    pub ws_url: Option<String>, // url of the websocket we're forwarding requests to.
    pub status: Status,         // stores stats related to the rpc.
    // For max_consecutive
    pub max_consecutive: u32, // max times we can call an rpc in a row
    pub consecutive: u32,
    // For max_per_second
    pub last_used: u128,      // last time we sent a querry to this node
    pub min_time_delta: u128, // microseconds
}

/// Sanitizes URLs so secrets don't get outputed.
///
/// For example, if we have a URL: https://eth-mainnet.g.alchemy.com/v2/api-key
// as input, we output: https://eth-mainnet.g.alchemy.com/
fn sanitize_url(url: &str) -> Result<String, url::ParseError> {
    let parsed_url = Url::parse(url)?;

    // Build a new URL with the scheme, host, and port (if any), but without the path or query
    let sanitized = Url::parse(&format!(
        "{}://{}{}",
        parsed_url.scheme(),
        parsed_url.host_str().unwrap_or_default(),
        match parsed_url.port() {
            Some(port) => format!(":{}", port),
            None => String::new(),
        }
    ))?;

    Ok(sanitized.to_string())
}

impl Default for Rpc {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            url: "".to_string(),
            ws_url: None,
            client: Client::new(),
            status: Status::default(),
            max_consecutive: 0,
            consecutive: 0,
            last_used: 0,
            min_time_delta: 0,
        }
    }
}

// implement new for rpc
impl Rpc {
    pub fn new(
        url: String,
        ws_url: Option<String>,
        max_consecutive: u32,
        min_time_delta: u128,
        ma_length: f64,
    ) -> Self {
        Self {
            name: sanitize_url(&url).unwrap_or(url.clone()),
            url,
            client: Client::new(),
            ws_url,
            status: Status {
                ma_length,
                ..Default::default()
            },
            max_consecutive,
            consecutive: 0,
            last_used: 0,
            min_time_delta,
        }
    }

    // Send requests using hyper
    pub async fn send_request(
        &self,
        parts: Parts,
        body_bytes: Bytes, /* other params */
    ) -> Result<(String, u16), Box<dyn std::error::Error>> {
        let allowed_headers = vec![
            "Content-Type".to_string(),
            "Authorization".to_string(),
            "access-control-allow-origin".to_string(),
        ];

        let path = parts.uri.path();
        let query = parts.uri.query().unwrap_or("");

        let url = if query.is_empty() {
            format!("{}{}", &self.url, path)
        } else {
            format!("{}{}?{}", &self.url, path, query)
        };
        println!("Received Request {:?}", url);

        // Create a new Hyper client
        let https = HttpsConnector::new();
        let client = HyperClient::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
        let mut filtered_headers = HeaderMap::new();
        for allowed_header in &allowed_headers {
            if let Some(value) = parts.headers.get(allowed_header) {
                if let Ok(name) = http::header::HeaderName::from_bytes(allowed_header.as_bytes()) {
                    filtered_headers.insert(name, value.clone());
                }
            }
        }

        // Extract the body and convert it to Full<Bytes>
        let full_body = Full::new(body_bytes);

        // Create a new request with the converted body
        let mut new_request = Request::from_parts(parts, full_body);
        *new_request.uri_mut() = url.parse()?; // Replace with your target server
        *new_request.headers_mut() = filtered_headers;

        let response = client.request(new_request).await?;
        let status = response.status().as_u16();
        // Convert the response body
        let body_bytes = response.collect().await?.to_bytes();
        let body_string = String::from_utf8(body_bytes.to_vec())?;
        Ok((body_string, status))
    }

    //function to send and get aptos rpc status response

    pub async fn send_request_aptos_health(&self) -> Result<String, crate::utils::error::RpcError> {
        send_health_request(self.url.clone(), self.client.clone()).await
    }


    /// Returns the sync status. False if we're synced and following the head.
    pub async fn syncing(&self) -> Result<bool, crate::utils::error::RpcError> {
        let sync = self.send_request_aptos_health().await?;
        // check response
        let is_valid = is_valid_api_response(&sync);
        // let status = extract_sync(&sync)?;

        Ok(is_valid)
    }

    /// Update the latency of the last n calls.
    /// We don't do it within send_request because we might kill it if it times out.
    pub fn update_latency(&mut self, latest: f64) {
        // If we have data >= to ma_length, remove the first one in line
        if self.status.latency_data.len() >= self.status.ma_length as usize {
            self.status.latency_data.remove(0);
        }

        // Update latency
        self.status.latency_data.push(latest);
        self.status.latency =
            self.status.latency_data.iter().sum::<f64>() / self.status.latency_data.len() as f64;
    }
}
