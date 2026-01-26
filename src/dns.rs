use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use tracing::info;

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    // Create resolver
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            ResolverOpts::default(),
        );
        
        Ok(Self { resolver })
    }
    
    // Resolve hostname
    pub async fn resolve(&self, domain: &str) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
        info!("Resolving: {}", domain);
        
        let response = self.resolver.lookup_ip(domain).await?;
        let ips: Vec<_> = response.iter().collect();
        
        info!("Resolved {} to {} addresses", domain, ips.len());
        
        Ok(ips)
    }
    // Multi-path resolve hook
    pub async fn multi_path_resolve(&self, domain: &str) -> Result<Vec<std::net::IpAddr>, Box<dyn std::error::Error>> {
        self.resolve(domain).await
    }
}
