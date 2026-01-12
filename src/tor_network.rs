use arti_client::{TorClient, TorClientConfig};
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
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Bootstrapping Tor connection...");
        
        // Create Tor client with default config
        let config = TorClientConfig::default();
        
        // Bootstrap connection to Tor network
        // This connects to directory servers and builds circuits
        let client = TorClient::create_bootstrapped(config).await?;
        
        info!("Tor bootstrapped! Connected to network.");
        
        Ok(Self {
            client: Arc::new(client),
        })
    }
    
    pub async fn route_request(
        &self,
        req: Request<hyper::body::Incoming>,
        fingerprint: &BrowserFingerprint,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let uri = req.uri().clone();
        let method = req.method().clone();
        
        info!("Routing {} {} through Tor", method, uri);
        
        // Extract host and port
        let host = uri.host().ok_or("No host in URI")?;
        let port = uri.port_u16().unwrap_or(if uri.scheme_str() == Some("https") { 443 } else { 80 });
        
        // Get path with query
        let path_and_query = uri.path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");
        
        info!("Connecting to {}:{} via Tor", host, port);
        
        // Connect through Tor
        let mut stream = self.client
            .connect((host, port))
            .await
            .map_err(|e| format!("Tor connection failed: {}", e))?;
        
        // Build proper HTTP/1.1 request with randomized fingerprint
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
        
        // Send through Tor stream
        use tokio::io::{AsyncWriteExt, AsyncReadExt};
        stream.write_all(request_data.as_bytes()).await?;
        stream.flush().await?;
        
        // Read response with timeout
        let mut response_bytes = Vec::new();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            stream.read_to_end(&mut response_bytes)
        ).await;
        
        match read_result {
            Ok(Ok(_)) => {
                info!("✓ Received response through Tor ({} bytes)", response_bytes.len());
                
                // Parse HTTP response
                let response_str = String::from_utf8_lossy(&response_bytes);
                
                // Split headers and body
                if let Some(body_start) = response_str.find("\r\n\r\n") {
                    let headers_part = &response_str[..body_start];
                    let body = &response_str[body_start + 4..];
                    
                    info!("Response headers: {}", headers_part.lines().next().unwrap_or("No status line"));
                    info!("Body length: {} bytes", body.len());
                    
                    Ok(Response::new(Full::new(Bytes::from(body.to_string()))))
                } else {
                    // No proper HTTP response, return raw data
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
    
    pub async fn check_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Test connection by fetching Tor check page
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
