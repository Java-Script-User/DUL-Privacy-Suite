use crate::config::Config;
use crate::routing::Router;
use crate::web_api::ApiState;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Bytes};
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error};

pub struct ProxyServer {
    config: Config,
    router: Router,
    app_state: Option<ApiState>,
}

impl ProxyServer {
    pub async fn new(config: Config, app_state: Option<ApiState>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let router = Router::new(config.clone(), app_state.clone()).await?;
        
        Ok(Self {
            config,
            router,
            app_state,
        })
    }
    
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: std::net::SocketAddr = self.config.proxy_addr().parse()?;
        let listener = TcpListener::bind(addr).await?;
        
        info!("Proxy server listening on {}", addr);
        
        // Spawn statistics reporter
        let stats_router = self.router.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let (blocklist_size, total_blocked) = stats_router.get_stats();
                info!("📊 Stats: {} trackers blocked this session (blocklist: {} domains)", total_blocked, blocklist_size);
            }
        });
        
        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    info!("🔌 New connection from: {}", client_addr);
                    
                    if let Some(ref state) = self.app_state {
                        state.add_log("info", format!("🔌 New connection from: {}", client_addr), "network").await;
                    }
                    
                    let router = self.router.clone();
                    let app_state = self.app_state.clone();
                    
                    tokio::spawn(async move {
                        // Read first line to check if it's CONNECT
                        let mut buffer = vec![0u8; 8192];
                        match stream.peek(&mut buffer).await {
                            Ok(n) if n > 0 => {
                                let request_start = String::from_utf8_lossy(&buffer[..n]);
                                
                                if request_start.starts_with("CONNECT ") {
                                    // Handle HTTPS tunnel
                                    if let Err(e) = handle_connect_tunnel(stream, router, app_state).await {
                                        error!("CONNECT tunnel error: {}", e);
                                    }
                                } else {
                                    // Handle regular HTTP with hyper
                                    let io = TokioIo::new(stream);
                                    
                                    let service = service_fn(move |req| {
                                        let router = router.clone();
                                        async move {
                                            handle_request(req, router).await
                                        }
                                    });
                                    
                                    if let Err(e) = http1::Builder::new()
                                        .serve_connection(io, service)
                                        .await
                                    {
                                        error!("Error serving connection: {}", e);
                                    }
                                }
                            }
                            _ => {
                                error!("Failed to peek stream data");
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connect_tunnel(
    mut client_stream: tokio::net::TcpStream,
    router: Router,
    app_state: Option<ApiState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read CONNECT request
    let mut buffer = vec![0u8; 8192];
    let n = client_stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);
    
    // Parse CONNECT target (e.g., "CONNECT example.com:443 HTTP/1.1")
    let first_line = request.lines().next().ok_or("Empty request")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err("Invalid CONNECT request".into());
    }
    
    let target = parts[1];
    info!("🔐 HTTPS tunnel request: {}", target);
    
    if let Some(ref state) = app_state {
        state.add_log("info", format!("🔐 HTTPS tunnel request: {}", target), "network").await;
        state.update_stats(|s| s.total_requests += 1).await;
    }
    
    // Parse host:port
    let host_port: Vec<&str> = target.split(':').collect();
    if host_port.len() != 2 {
        return Err("Invalid host:port in CONNECT".into());
    }
    
    let host = host_port[0];
    let port: u16 = host_port[1].parse()?;
    
    // Connect through Tor
    let tor_stream = router.connect_through_tor(host, port).await?;
    
    // Send success response to client
    client_stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
    client_stream.flush().await?;
    
    info!("✅ HTTPS tunnel established to {} via Tor", target);
    
    if let Some(ref state) = app_state {
        state.add_log("info", format!("✅ HTTPS tunnel established to {} via Tor", target), "network").await;
    }
    
    // Start bidirectional copy
    let (mut client_read, mut client_write) = client_stream.split();
    let (mut tor_read, mut tor_write) = tokio::io::split(tor_stream);
    
    let client_to_tor = tokio::io::copy(&mut client_read, &mut tor_write);
    let tor_to_client = tokio::io::copy(&mut tor_read, &mut client_write);
    
    // Run both directions concurrently
    tokio::select! {
        result = client_to_tor => {
            if let Err(e) = result {
                error!("Client->Tor copy error: {}", e);
            }
        }
        result = tor_to_client => {
            if let Err(e) = result {
                error!("Tor->Client copy error: {}", e);
            }
        }
    }
    
    info!("🔌 HTTPS tunnel closed: {}", target);
    
    if let Some(ref state) = app_state {
        state.add_log("info", format!("🔌 HTTPS tunnel closed: {}", target), "network").await;
    }
    
    Ok(())
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    router: Router,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    info!("📡 HTTP Request: {} {}", method, uri);
    
    // Route through multi-hop network
    match router.route_request(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Routing error: {}", e);
            Ok(Response::new(Full::new(Bytes::from("Error processing request"))))
        }
    }
}
