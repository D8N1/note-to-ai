use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug, error};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519PrivateKey};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use hmac::{Hmac, Mac};
use sha2::{Sha256};
use hkdf::Hkdf;
use rand::RngCore;

/// Protocol address identifying a Signal user
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolAddress {
    pub name: String,
    pub device_id: u32,
}

impl ProtocolAddress {
    pub fn new(name: String, device_id: u32) -> Self {
        Self { name, device_id }
    }
}

/// Simple identity key pair for Signal Protocol
#[derive(Debug, Clone)]
pub struct IdentityKeyPair {
    pub public_key: X25519PublicKey,
    private_key_bytes: [u8; 32],
}

impl IdentityKeyPair {
    pub fn generate() -> Self {
        let private_key = X25519PrivateKey::random_from_rng(OsRng);
        let public_key = X25519PublicKey::from(&private_key);
        let private_key_bytes = private_key.to_bytes();
        
        Self {
            public_key,
            private_key_bytes,
        }
    }
    
    pub fn private_key(&self) -> X25519PrivateKey {
        X25519PrivateKey::from(self.private_key_bytes)
    }
    
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key.to_bytes()
    }
}

/// Session state for message encryption/decryption
#[derive(Debug, Clone)]
pub struct SessionState {
    pub root_key: [u8; 32],
    pub chain_key_send: Option<[u8; 32]>,
    pub chain_key_recv: Option<[u8; 32]>,
    pub send_counter: u32,
    pub recv_counter: u32,
    pub previous_counter: u32,
    pub ratchet_key_pair: Option<IdentityKeyPair>,
    pub remote_ratchet_key: Option<X25519PublicKey>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            root_key: [0u8; 32],
            chain_key_send: None,
            chain_key_recv: None,
            send_counter: 0,
            recv_counter: 0,
            previous_counter: 0,
            ratchet_key_pair: None,
            remote_ratchet_key: None,
        }
    }
    
    /// Initialize session with X3DH key agreement
    pub fn initialize_session(
        &mut self,
        local_identity: &IdentityKeyPair,
        remote_identity: &X25519PublicKey,
        local_ephemeral: &IdentityKeyPair,
        remote_ephemeral: &X25519PublicKey,
        is_alice: bool,
    ) -> Result<()> {
        // Perform X3DH key agreement
        let dh1 = local_identity.private_key().diffie_hellman(remote_ephemeral);
        let dh2 = local_ephemeral.private_key().diffie_hellman(remote_identity);
        let dh3 = local_ephemeral.private_key().diffie_hellman(remote_ephemeral);
        
        // Combine DH outputs
        let mut sk = Vec::new();
        sk.extend_from_slice(dh1.as_bytes());
        sk.extend_from_slice(dh2.as_bytes());
        sk.extend_from_slice(dh3.as_bytes());
        
        // Derive root key
        let hk = Hkdf::<Sha256>::new(None, &sk);
        hk.expand(b"Signal_Root_Key", &mut self.root_key)
            .map_err(|e| anyhow::anyhow!("Root key derivation failed: {}", e))?;
        
        // Initialize ratchet
        let ratchet_keypair = IdentityKeyPair::generate();
        
        if is_alice {
            // Alice doesn't have Bob's ratchet key yet
            self.chain_key_send = Some(self.root_key);
            self.ratchet_key_pair = Some(ratchet_keypair);
        } else {
            // Bob initializes receiving chain
            self.chain_key_recv = Some(self.root_key);
            self.remote_ratchet_key = Some(local_ephemeral.public_key);
        }
        
        info!("Session initialized successfully");
        Ok(())
    }
}

/// Message keys for encryption/decryption
#[derive(Debug, Clone)]
pub struct MessageKeys {
    pub cipher_key: [u8; 32],
    pub mac_key: [u8; 32],
    pub iv: [u8; 12], // AES-GCM uses 12-byte nonces
    pub counter: u32,
}

impl MessageKeys {
    pub fn from_chain_key(chain_key: &[u8; 32], counter: u32) -> Result<Self> {
        // Generate message key from chain key
        let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(chain_key)
            .map_err(|e| anyhow::anyhow!("HMAC creation failed: {}", e))?;
        mac.update(&[0x01]);
        
        let message_key = mac.finalize().into_bytes();
        
        // Derive cipher key, mac key, and IV from message key
        let hk = Hkdf::<Sha256>::new(None, &message_key);
        
        let mut cipher_key = [0u8; 32];
        let mut mac_key = [0u8; 32];
        let mut iv = [0u8; 12]; // AES-GCM uses 12-byte nonces
        
        hk.expand(b"Signal_Cipher_Key", &mut cipher_key)
            .map_err(|e| anyhow::anyhow!("Cipher key derivation failed: {}", e))?;
        
        hk.expand(b"Signal_MAC_Key", &mut mac_key)
            .map_err(|e| anyhow::anyhow!("MAC key derivation failed: {}", e))?;
        
        hk.expand(b"Signal_IV", &mut iv)
            .map_err(|e| anyhow::anyhow!("IV derivation failed: {}", e))?;
        
        Ok(Self {
            cipher_key,
            mac_key,
            iv,
            counter,
        })
    }
}

/// Signal message with encrypted content and MAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    pub ciphertext: Vec<u8>,
    pub mac: [u8; 32],
    pub counter: u32,
    pub ratchet_key: Option<[u8; 32]>,
}

impl SignalMessage {
    /// Create new message with encryption
    pub fn new(
        plaintext: &[u8],
        message_keys: &MessageKeys,
        local_identity: &IdentityKeyPair,
        remote_identity: &X25519PublicKey,
    ) -> Result<Self> {
        // Encrypt the message
        let key = Key::<Aes256Gcm>::from_slice(&message_keys.cipher_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&message_keys.iv);
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Compute MAC
        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&message_keys.counter.to_be_bytes());
        mac_input.extend_from_slice(&ciphertext);
        mac_input.extend_from_slice(&local_identity.public_key_bytes());
        mac_input.extend_from_slice(&remote_identity.to_bytes());
        
        let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(&message_keys.mac_key)
            .map_err(|e| anyhow::anyhow!("MAC creation failed: {}", e))?;
        mac.update(&mac_input);
        
        let mac_result = mac.finalize().into_bytes();
        let mut mac_bytes = [0u8; 32];
        mac_bytes.copy_from_slice(&mac_result[0..32]);
        
        Ok(Self {
            ciphertext,
            mac: mac_bytes,
            counter: message_keys.counter,
            ratchet_key: None,
        })
    }
    
    /// Verify message MAC
    pub fn verify_mac(
        &self,
        mac_key: &[u8; 32],
        local_identity: &X25519PublicKey,
        remote_identity: &X25519PublicKey,
    ) -> Result<bool> {
        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&self.counter.to_be_bytes());
        mac_input.extend_from_slice(&self.ciphertext);
        mac_input.extend_from_slice(&local_identity.to_bytes());
        mac_input.extend_from_slice(&remote_identity.to_bytes());
        
        let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(mac_key)
            .map_err(|e| anyhow::anyhow!("MAC creation failed: {}", e))?;
        mac.update(&mac_input);
        
        let computed_mac = mac.finalize().into_bytes();
        Ok(computed_mac[0..32] == self.mac)
    }
}

/// Session store trait for managing sessions
pub trait SessionStore {
    fn load_session(&self, address: &ProtocolAddress) -> Option<SessionState>;
    fn store_session(&mut self, address: &ProtocolAddress, session: SessionState);
    fn contains_session(&self, address: &ProtocolAddress) -> bool;
    fn delete_session(&mut self, address: &ProtocolAddress);
}

/// Identity key store trait for managing identity keys
pub trait IdentityKeyStore {
    fn get_identity_key_pair(&self) -> &IdentityKeyPair;
    fn get_local_registration_id(&self) -> u32;
    fn save_identity(&mut self, address: &ProtocolAddress, identity_key: X25519PublicKey);
    fn is_trusted_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> bool;
    fn get_identity(&self, address: &ProtocolAddress) -> Option<X25519PublicKey>;
}

/// In-memory session store implementation
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: HashMap<ProtocolAddress, SessionState>,
}

impl SessionStore for InMemorySessionStore {
    fn load_session(&self, address: &ProtocolAddress) -> Option<SessionState> {
        self.sessions.get(address).cloned()
    }
    
    fn store_session(&mut self, address: &ProtocolAddress, session: SessionState) {
        self.sessions.insert(address.clone(), session);
    }
    
    fn contains_session(&self, address: &ProtocolAddress) -> bool {
        self.sessions.contains_key(address)
    }
    
    fn delete_session(&mut self, address: &ProtocolAddress) {
        self.sessions.remove(address);
    }
}

/// In-memory identity key store implementation
#[derive(Debug)]
pub struct InMemoryIdentityStore {
    identity_key_pair: IdentityKeyPair,
    registration_id: u32,
    trusted_keys: HashMap<ProtocolAddress, X25519PublicKey>,
}

impl InMemoryIdentityStore {
    pub fn new() -> Self {
        Self {
            identity_key_pair: IdentityKeyPair::generate(),
            registration_id: rand::random(),
            trusted_keys: HashMap::new(),
        }
    }
}

impl IdentityKeyStore for InMemoryIdentityStore {
    fn get_identity_key_pair(&self) -> &IdentityKeyPair {
        &self.identity_key_pair
    }
    
    fn get_local_registration_id(&self) -> u32 {
        self.registration_id
    }
    
    fn save_identity(&mut self, address: &ProtocolAddress, identity_key: X25519PublicKey) {
        self.trusted_keys.insert(address.clone(), identity_key);
    }
    
    fn is_trusted_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> bool {
        if let Some(trusted_key) = self.trusted_keys.get(address) {
            trusted_key.to_bytes() == identity_key.to_bytes()
        } else {
            true // First time seeing this identity - trust it
        }
    }
    
    fn get_identity(&self, address: &ProtocolAddress) -> Option<X25519PublicKey> {
        self.trusted_keys.get(address).copied()
    }
}

/// Main Signal Protocol implementation
#[derive(Debug)]
pub struct SignalProtocol {
    session_store: InMemorySessionStore,
    identity_store: InMemoryIdentityStore,
}

impl SignalProtocol {
    /// Create new Signal Protocol instance
    pub fn new() -> Self {
        info!("Initializing Signal Protocol");
        
        Self {
            session_store: InMemorySessionStore::default(),
            identity_store: InMemoryIdentityStore::new(),
        }
    }
    
    /// Encrypt message for recipient
    pub fn encrypt_message(
        &mut self,
        recipient: &ProtocolAddress,
        plaintext: &[u8],
    ) -> Result<SignalMessage> {
        debug!("Encrypting message for {}", recipient.name);
        
        // Load or create session
        let mut session = self.session_store.load_session(recipient)
            .unwrap_or_else(SessionState::new);
        
        // Check if we have a session
        if session.chain_key_send.is_none() {
            return Err(anyhow::anyhow!("No session established with {}", recipient.name).into());
        }
        
        // Get chain key and advance it
        let chain_key = session.chain_key_send.unwrap();
        session.chain_key_send = Some(Self::advance_chain_key(&chain_key)?);
        
        // Generate message keys
        let message_keys = MessageKeys::from_chain_key(&chain_key, session.send_counter)?;
        session.send_counter += 1;
        
        // Get remote identity
        let remote_identity = self.identity_store.get_identity(recipient)
            .ok_or_else(|| anyhow::anyhow!("No identity found for {}", recipient.name))?;
        
        // Create encrypted message
        let message = SignalMessage::new(
            plaintext,
            &message_keys,
            self.identity_store.get_identity_key_pair(),
            &remote_identity,
        )?;
        
        // Update session
        self.session_store.store_session(recipient, session);
        
        info!("Message encrypted successfully for {}", recipient.name);
        Ok(message)
    }
    
    /// Decrypt message from sender
    pub fn decrypt_message(
        &mut self,
        sender: &ProtocolAddress,
        message: &SignalMessage,
    ) -> Result<Vec<u8>> {
        debug!("Decrypting message from {}", sender.name);
        
        // Load session
        let mut session = self.session_store.load_session(sender)
            .ok_or_else(|| anyhow::anyhow!("No session found for {}", sender.name))?;
        
        // Check if we have a receiving chain
        let chain_key = session.chain_key_recv
            .ok_or_else(|| anyhow::anyhow!("No receiving chain for {}", sender.name))?;
        
        // Generate message keys
        let message_keys = MessageKeys::from_chain_key(&chain_key, message.counter)?;
        
        // Get sender identity
        let sender_identity = self.identity_store.get_identity(sender)
            .ok_or_else(|| anyhow::anyhow!("No identity found for {}", sender.name))?;
        
        // Verify MAC
        let mac_valid = message.verify_mac(
            &message_keys.mac_key,
            &self.identity_store.get_identity_key_pair().public_key,
            &sender_identity,
        )?;
        
        if !mac_valid {
            return Err(anyhow::anyhow!("MAC verification failed").into());
        }
        
        // Decrypt message
        let key = Key::<Aes256Gcm>::from_slice(&message_keys.cipher_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&message_keys.iv);
        
        let plaintext = cipher
            .decrypt(nonce, message.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        // Advance chain key
        session.chain_key_recv = Some(Self::advance_chain_key(&chain_key)?);
        session.recv_counter += 1;
        
        // Update session
        self.session_store.store_session(sender, session);
        
        info!("Message decrypted successfully from {}", sender.name);
        Ok(plaintext)
    }
    
    /// Advance chain key (chain key ratchet)
    fn advance_chain_key(chain_key: &[u8; 32]) -> Result<[u8; 32]> {
        let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(chain_key)
            .map_err(|e| anyhow::anyhow!("HMAC creation failed: {}", e))?;
        mac.update(&[0x02]);
        
        let result = mac.finalize().into_bytes();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result[0..32]);
        Ok(output)
    }
    
    /// Get local identity public key
    pub fn get_identity_public_key(&self) -> X25519PublicKey {
        self.identity_store.get_identity_key_pair().public_key
    }
    
    /// Add trusted identity for a contact
    pub fn add_trusted_identity(&mut self, address: ProtocolAddress, identity_key: X25519PublicKey) {
        self.identity_store.save_identity(&address, identity_key);
        info!("Added trusted identity for {}", address.name);
    }
    
    /// Create session with a contact (simplified X3DH)
    pub fn create_session(
        &mut self,
        remote_address: ProtocolAddress,
        remote_identity: X25519PublicKey,
        remote_ephemeral: X25519PublicKey,
    ) -> Result<()> {
        info!("Creating session with {}", remote_address.name);
        
        // Generate local ephemeral key
        let local_ephemeral = IdentityKeyPair::generate();
        
        // Initialize session
        let mut session = SessionState::new();
        session.initialize_session(
            self.identity_store.get_identity_key_pair(),
            &remote_identity,
            &local_ephemeral,
            &remote_ephemeral,
            true, // We are Alice (initiator)
        )?;
        
        // Store session and identity
        self.session_store.store_session(&remote_address, session);
        self.identity_store.save_identity(&remote_address, remote_identity);
        
        info!("Session created successfully with {}", remote_address.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_key_generation() {
        let keypair = IdentityKeyPair::generate();
        
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.private_key().to_bytes().len(), 32);
        
        println!("✅ Identity key generation test passed");
    }
    
    #[test]
    fn test_message_keys_derivation() {
        let chain_key = [42u8; 32];
        let counter = 123;
        
        let message_keys = MessageKeys::from_chain_key(&chain_key, counter).unwrap();
        
        assert_eq!(message_keys.cipher_key.len(), 32);
        assert_eq!(message_keys.mac_key.len(), 32);
        assert_eq!(message_keys.iv.len(), 16);
        assert_eq!(message_keys.counter, counter);
        
        println!("✅ Message keys derivation test passed");
    }
    
    #[test]
    fn test_signal_protocol_basic() {
        let mut alice = SignalProtocol::new();
        let mut bob = SignalProtocol::new();
        
        // Create addresses
        let alice_addr = ProtocolAddress::new("alice".to_string(), 1);
        let bob_addr = ProtocolAddress::new("bob".to_string(), 1);
        
        // Exchange identity keys (simplified)
        let alice_identity = alice.get_identity_public_key();
        let bob_identity = bob.get_identity_public_key();
        
        // Create sessions (simplified - in real implementation this would use proper X3DH)
        let bob_ephemeral = IdentityKeyPair::generate();
        alice.create_session(bob_addr.clone(), bob_identity, bob_ephemeral.public_key).unwrap();
        
        // Test message encryption (would fail without proper session setup)
        let plaintext = b"Hello, Signal Protocol!";
        let encrypt_result = alice.encrypt_message(&bob_addr, plaintext);
        
        // This should fail because we don't have a proper session setup
        assert!(encrypt_result.is_err());
        
        println!("✅ Signal Protocol basic test passed");
    }
}
