use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub address: String,
    pub public_key: Option<Vec<u8>>,
    pub reputation: f32,
    pub latency_ms: Option<u64>,
}

impl Node {
    // Create a node entry
    pub fn new(address: String) -> Self {
        Self {
            address,
            public_key: None,
            reputation: 1.0,
            latency_ms: None,
        }
    }
    
    pub async fn ping(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(50)
    }
    
    pub fn is_available(&self) -> bool {
        self.reputation > 0.5
    }
}

pub struct NodeRegistry {
    db: sled::Db,
}

impl NodeRegistry {
    // Open node registry DB
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = sled::open(db_path)?;
        Ok(Self { db })
    }
    
    // Store a node
    pub fn add_node(&self, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        let key = node.address.as_bytes();
        let value = serde_json::to_vec(node)?;
        self.db.insert(key, value)?;
        Ok(())
    }
    
    // Load available nodes
    pub fn get_all_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let mut nodes = Vec::new();
        for item in self.db.iter() {
            let (_key, value) = item?;
            let node: Node = serde_json::from_slice(&value)?;
            if node.is_available() {
                nodes.push(node);
            }
        }
        
        Ok(nodes)
    }
}
