use arti_client::{TorClient, TorClientConfig};
use arti_client::config::{BridgeConfigBuilder, CfgPath};
use arti_client::config::pt::TransportConfigBuilder;
use hyper::{Request, Response, body::Bytes};
use http_body_util::Full;
use tracing::{info, error};
use std::sync::Arc;
use crate::fingerprint::BrowserFingerprint;

#[derive(Clone)]
pub struct TorNetwork {
    client: Arc<TorClient<tor_rtcompat::PreferredRuntime>>,
}

impl TorNetwork {
    // Bootstrap Tor client
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Bootstrapping Tor connection...");
        let config = TorClientConfig::default();
        match TorClient::create_bootstrapped(config).await {
            Ok(client) => {
                info!("Tor bootstrapped! Connected to network.");
                Ok(Self { client: Arc::new(client) })
            }
            Err(e) => {
                error!("Tor bootstrap failed: {}", e);
                info!("Trying obfs4 bridge fallback...");
                let fallback = Self::build_obfs4_config()?;
                let client = TorClient::create_bootstrapped(fallback).await?;
                info!("Tor bootstrapped via obfs4 bridges.");
                Ok(Self { client: Arc::new(client) })
            }
        }
    }

    fn build_obfs4_config() -> Result<TorClientConfig, Box<dyn std::error::Error + Send + Sync>> {
        let mut builder = TorClientConfig::builder();

        const OBFS4_BRIDGE_1: &str = "Bridge obfs4 95.216.37.112:59745 10809FCDC7B96F2C40AEACE3BF2497DA76B6A494 cert=8ainAYxaoJKgDFy7kFJ3DN6WO/YmsSfkjvqanv5UDQD0NE3PPh6igcFWu/z40sK2rHquGg iat-mode=0";
        const OBFS4_BRIDGE_2: &str = "Bridge obfs4 139.144.209.47:8000 02E4E04C425EA273FE248E432758F8370101F1DB cert=j9N4ICLlx5Mj6xNi5yzqUcDvd4bOHu7fJ0F/Ev/7DlNK/MmC8pAgK7d2LLWpJ2SX/jyLYQ iat-mode=0";
        let bridge1: BridgeConfigBuilder = OBFS4_BRIDGE_1.parse()?;
        let bridge2: BridgeConfigBuilder = OBFS4_BRIDGE_2.parse()?;
        builder.bridges().bridges().push(bridge1);
        builder.bridges().bridges().push(bridge2);

        let lyrebird_path = std::env::var("LYREBIRD_PATH")
            .ok()
            .filter(|p| !p.trim().is_empty())
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("lyrebird.exe")))
                    .filter(|p| p.exists())
                    .map(|p| p.display().to_string())
            })
            .unwrap_or_else(|| "lyrebird".to_string());

        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["obfs4".parse()?])
            .path(CfgPath::new(lyrebird_path.into()))
            .run_on_startup(true);
        builder.bridges().transports().push(transport);

        Ok(builder.build()?)
    }
    
    // Issue a direct HTTP request over Tor
    pub async fn route_request(
        &self,
        req: Request<hyper::body::Incoming>,
        fingerprint: &BrowserFingerprint,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let uri = req.uri().clone();
        let method = req.method().clone();
        
        info!("Routing {} {} through Tor", method, uri);
        
        let host = uri.host().ok_or("No host in URI")?;
        let port = uri.port_u16().unwrap_or(if uri.scheme_str() == Some("https") { 443 } else { 80 });
        let path_and_query = uri.path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");
        info!("Connecting to {}:{} via Tor", host, port);
        let mut stream = self.client
            .connect((host, port))
            .await
            .map_err(|e| format!("Tor connection failed: {}", e))?;
        let request_data = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: {}\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\nAccept-Language: {}\r\nAccept-Encoding: {}\r\nConnection: close\r\n\r\n",
            method,
            path_and_query,
            host,
            fingerprint.user_agent,
            fingerprint.accept_language,
            fingerprint.accept_encoding
        );
        info!("Sending request through Tor circuit...");
        use tokio::io::{AsyncWriteExt, AsyncReadExt};
        stream.write_all(request_data.as_bytes()).await?;
        stream.flush().await?;
        let mut response_bytes = Vec::new();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            stream.read_to_end(&mut response_bytes)
        ).await;
        
        match read_result {
            Ok(Ok(_)) => {
                info!("✓ Received response through Tor ({} bytes)", response_bytes.len());
                let response_str = String::from_utf8_lossy(&response_bytes);
                if let Some(body_start) = response_str.find("\r\n\r\n") {
                    let headers_part = &response_str[..body_start];
                    let body = &response_str[body_start + 4..];
                    info!("Response headers: {}", headers_part.lines().next().unwrap_or("No status line"));
                    info!("Body length: {} bytes", body.len());
                    Ok(Response::new(Full::new(Bytes::from(body.to_string()))))
                } else {
                    Ok(Response::new(Full::new(Bytes::from(response_str.to_string()))))
                }
            }
            Ok(Err(e)) => {
                Err(format!("Failed to read response: {}", e).into())
            }
            Err(_) => {
                Err("Request timeout after 30 seconds".into())
            }
        }
    }
    
    // Open a raw Tor stream
    pub async fn connect_stream(
        &self,
        host: &str,
        port: u16,
    ) -> Result<arti_client::DataStream, Box<dyn std::error::Error + Send + Sync>> {
        info!("Establishing Tor stream to {}:{}", host, port);
        
        let stream = self.client
            .connect((host, port))
            .await
            .map_err(|e| format!("Tor stream connection failed: {}", e))?;
        
        Ok(stream)
    }
    
    // Simple Tor connectivity check
    pub async fn check_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        info!("Testing Tor connection...");
        let test_stream = self.client
            .connect(("check.torproject.org", 443))
            .await;
        
        match test_stream {
            Ok(_) => {
                info!("✓ Tor connection working!");
                Ok(true)
            }
            Err(e) => {
                error!("✗ Tor connection failed: {}", e);
                Ok(false)
            }
        }
    }
}
