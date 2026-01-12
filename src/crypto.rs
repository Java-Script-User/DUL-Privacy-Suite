use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use x25519_dalek::{EphemeralSecret, PublicKey};
use crate::network::Node;

#[derive(Clone)]
pub struct CryptoLayer {
    // Private key for this session
    session_key: Vec<u8>,
}

impl CryptoLayer {
    pub fn new() -> Self {
        // Generate session key
        let mut key = vec![0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut key);
        
        Self {
            session_key: key,
        }
    }
    
    /// Build encrypted onion layers for multi-hop routing
    pub fn build_onion_layers(
        &self,
        req: hyper::Request<hyper::body::Incoming>,
        route: &[&Node],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement proper onion routing encryption
        // Each layer encrypts the next hop's address and payload
        
        // For now, serialize the request
        let uri = req.uri().to_string();
        let method = req.method().to_string();
        
        let payload = format!("{}::{}", method, uri);
        
        // Encrypt with each node's key in reverse order
        let mut encrypted = payload.into_bytes();
        
        for node in route.iter().rev() {
            encrypted = self.encrypt_layer(&encrypted, node)?;
        }
        
        Ok(encrypted)
    }
    
    fn encrypt_layer(
        &self,
        data: &[u8],
        _node: &Node,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Use node's public key for encryption
        // For now, use session key
        
        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)?;
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| format!("Encryption error: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt_layer(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if data.len() < 12 {
            return Err("Invalid encrypted data".into());
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)?;
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;
        
        Ok(plaintext)
    }
}

/// Generate X25519 key pair for node identity
pub fn generate_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret, public)
}
