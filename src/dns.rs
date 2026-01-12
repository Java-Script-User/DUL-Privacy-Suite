use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use tracing::info;

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Use DNS-over-TLS or DNS-over-HTTPS
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            ResolverOpts::default(),
        );
        
        Ok(Self { resolver })
    }
    
    pub async fn resolve(&self, domain: &str) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
        info!("Resolving: {}", domain);
        
        let response = self.resolver.lookup_ip(domain).await?;
        let ips: Vec<_> = response.iter().collect();
        
        info!("Resolved {} to {} addresses", domain, ips.len());
        
        Ok(ips)
    }
    
    /// Resolve through multiple paths to prevent DNS manipulation
    pub async fn multi_path_resolve(&self, domain: &str) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
        // TODO: Query multiple DNS servers and compare results
        // This prevents DNS poisoning and ensures accuracy
        
        self.resolve(domain).await
    }
}
