use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub proxy_addr: String,
    pub num_hops: usize,
    pub dns_servers: Vec<String>,
    pub fingerprint_protection: bool,
    pub tracker_lists: Vec<String>,
    pub blockchain: BlockchainConfig,
    pub node_db_path: String,
    #[serde(skip)]
    config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub eth_rpc: String,
    pub payment_contract: String,
    pub wallet_address: Option<String>,
}

impl Config {
    // Load from disk or create defaults
    pub fn load_or_create() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: Config = toml::from_str(&content)?;
            config.config_path = config_path;
            Ok(config)
        } else {
            fs::create_dir_all(&config_dir)?;
            let config = Self::default_with_path(config_path.clone());
            let toml_str = toml::to_string_pretty(&config)?;
            fs::write(&config_path, toml_str)?;
            Ok(config)
        }
    }
    
    // Config directory under home
    fn config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir()
            .ok_or("Could not determine home directory")?;
        Ok(home.join(".privacy_suite"))
    }
    
    // Default config with saved path
    fn default_with_path(path: PathBuf) -> Self {
        let mut config = Self::default();
        config.config_path = path;
        config
    }
    
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
    
    pub fn proxy_addr(&self) -> &str {
        &self.proxy_addr
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proxy_addr: "0.0.0.0:8888".to_string(),
            num_hops: 3,
            dns_servers: vec![
                "1.1.1.1:853".to_string(),
                "8.8.8.8:853".to_string(),
            ],
            fingerprint_protection: true,
            tracker_lists: vec![
                "https://easylist.to/easylist/easylist.txt".to_string(),
                "https://easylist.to/easylist/easyprivacy.txt".to_string(),
            ],
            blockchain: BlockchainConfig {
                eth_rpc: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
                payment_contract: "0x0000000000000000000000000000000000000000".to_string(),
                wallet_address: None,
            },
            node_db_path: "~/.privacy_suite/nodes.db".to_string(),
            config_path: PathBuf::new(),
        }
    }
}
