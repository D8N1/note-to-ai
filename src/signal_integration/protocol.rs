use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519PrivateKey};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use rand::{rngs::OsRng, RngCore, CryptoRng};

/// Protocol address for Signal messaging
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolAddress {
    pub name: String,
    pub device_id: u32,
}

impl ProtocolAddress {
    pub fn new(name: String, device_id: u32) -> Self {
        Self { name, device_id }
    }
}

/// Identity key pair for Signal protocol
#[derive(Debug, Clone)]
pub struct IdentityKeyPair {
    pub public_key: X25519PublicKey,
    // Store private key as bytes to avoid Debug trait issues
    private_key_bytes: [u8; 32],
}

impl IdentityKeyPair {
    pub fn generate() -> Self {
        let private_key = X25519PrivateKey::random_from_rng(OsRng);
        let public_key = X25519PublicKey::from(&private_key);
        let private_key_bytes = private_key.to_bytes();
        Self { private_key_bytes, public_key }
    }
    
    pub fn private_key(&self) -> X25519PrivateKey {
        X25519PrivateKey::from(self.private_key_bytes)
    }
    
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.private_key_bytes
    }
    
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key.to_bytes()
    }
}

/// Session state for the double ratchet
#[derive(Debug, Clone)]
pub struct SessionState {
    pub root_key: [u8; 32],
    pub chain_key_send: Option<[u8; 32]>,
    pub chain_key_recv: Option<[u8; 32]>,
    pub send_counter: u32,
    pub recv_counter: u32,
    pub previous_send_counter: u32,
    pub ratchet_key_pair: Option<IdentityKeyPair>,
    pub remote_ratchet_key: Option<X25519PublicKey>,
    pub remote_identity_key: Option<X25519PublicKey>,
    pub local_identity_key: X25519PublicKey,
    pub session_version: u32,
    pub alice_base_key: Option<X25519PublicKey>,
}

impl SessionState {
    pub fn new(local_identity: X25519PublicKey) -> Self {
        Self {
            root_key: [0u8; 32],
            chain_key_send: None,
            chain_key_recv: None,
            send_counter: 0,
            recv_counter: 0,
            previous_send_counter: 0,
            ratchet_key_pair: None,
            remote_ratchet_key: None,
            remote_identity_key: None,
            local_identity_key: local_identity,
            session_version: 4, // Signal protocol version 4
            alice_base_key: None,
        }
    }
    
    /// Initialize Alice's session (initiator)
    pub fn initialize_alice(
        &mut self,
        bob_identity_key: X25519PublicKey,
        bob_ephemeral_key: X25519PublicKey,
        alice_identity_key: &IdentityKeyPair,
        alice_ephemeral_key: &IdentityKeyPair,
    ) -> Result<()> {
        self.remote_identity_key = Some(bob_identity_key);
        self.alice_base_key = Some(alice_ephemeral_key.public_key);
        
        // X3DH key agreement
        let shared_secrets = self.x3dh_alice(
            alice_identity_key,
            alice_ephemeral_key,
            bob_identity_key,
            bob_ephemeral_key,
        )?;
        
        // Initialize root key from X3DH output
        self.root_key = shared_secrets;
        
        // Initialize sending ratchet
        self.ratchet_key_pair = Some(IdentityKeyPair::generate());
        self.update_root_and_chain_keys_send()?;
        
        info!("Alice session initialized successfully");
        Ok(())
    }
    
    /// Initialize Bob's session (responder)
    pub fn initialize_bob(
        &mut self,
        alice_identity_key: X25519PublicKey,
        alice_ephemeral_key: X25519PublicKey,
        bob_identity_key: &IdentityKeyPair,
        bob_ephemeral_key: &IdentityKeyPair,
    ) -> Result<()> {
        self.remote_identity_key = Some(alice_identity_key);
        self.alice_base_key = Some(alice_ephemeral_key);
        
        // X3DH key agreement
        let shared_secrets = self.x3dh_bob(
            bob_identity_key,
            bob_ephemeral_key,
            alice_identity_key,
            alice_ephemeral_key,
        )?;
        
        // Initialize root key from X3DH output
        self.root_key = shared_secrets;
        
        info!("Bob session initialized successfully");
        Ok(())
    }
    
    /// X3DH key agreement for Alice (initiator)
    fn x3dh_alice(
        &self,
        alice_identity: &IdentityKeyPair,
        alice_ephemeral: &IdentityKeyPair,
        bob_identity: X25519PublicKey,
        bob_ephemeral: X25519PublicKey,
    ) -> Result<[u8; 32]> {
        // DH1 = DH(IK_A, SPK_B)
        let dh1 = alice_identity.private_key().diffie_hellman(&bob_ephemeral);
        
        // DH2 = DH(EK_A, IK_B)  
        let dh2 = alice_ephemeral.private_key().diffie_hellman(&bob_identity);
        
        // DH3 = DH(EK_A, SPK_B)
        let dh3 = alice_ephemeral.private_key().diffie_hellman(&bob_ephemeral);
        
        // SK = KDF(DH1 || DH2 || DH3)
        let mut sk_input = Vec::new();
        sk_input.extend_from_slice(dh1.as_bytes());
        sk_input.extend_from_slice(dh2.as_bytes());
        sk_input.extend_from_slice(dh3.as_bytes());
        
        let hk = Hkdf::<Sha256>::new(None, &sk_input);
        let mut output = [0u8; 32];
        hk.expand(b"Signal_X3DH", &mut output)
            .map_err(|e| anyhow::anyhow!("HKDF expansion failed: {}", e))?;
        
        debug!("X3DH key agreement completed (Alice)");
        Ok(output)
    }
    
    /// X3DH key agreement for Bob (responder)
    fn x3dh_bob(
        &self,
        bob_identity: &IdentityKeyPair,
        bob_ephemeral: &IdentityKeyPair,
        alice_identity: X25519PublicKey,
        alice_ephemeral: X25519PublicKey,
    ) -> Result<[u8; 32]> {
        // DH1 = DH(SPK_B, IK_A)
        let dh1 = bob_ephemeral.private_key().diffie_hellman(&alice_identity.public_key);
        
        // DH2 = DH(IK_B, EK_A)
        let dh2 = bob_identity.private_key().diffie_hellman(&alice_ephemeral);
        
        // DH3 = DH(SPK_B, EK_A)
        let dh3 = bob_ephemeral.private_key.diffie_hellman(&alice_ephemeral);
        
        // SK = KDF(DH1 || DH2 || DH3)
        let mut sk_input = Vec::new();
        sk_input.extend_from_slice(dh1.as_bytes());
        sk_input.extend_from_slice(dh2.as_bytes());
        sk_input.extend_from_slice(dh3.as_bytes());
        
        let hk = Hkdf::<Sha256>::new(None, &sk_input);
        let mut output = [0u8; 32];
        hk.expand(b"Signal_X3DH", &mut output)
            .map_err(|e| anyhow::anyhow!("HKDF expansion failed: {}", e))?;
        
        debug!("X3DH key agreement completed (Bob)");
        Ok(output)
    }
    
    /// Update root key and sending chain key (Double Ratchet step)
    fn update_root_and_chain_keys_send(&mut self) -> Result<()> {
        let ratchet_key = self.ratchet_key_pair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No ratchet key pair"))?;
        
        let remote_ratchet = self.remote_ratchet_key
            .ok_or_else(|| anyhow::anyhow!("No remote ratchet key"))?;
        
        let dh_out = ratchet_key.private_key.diffie_hellman(&remote_ratchet);
        
        // Root key ratchet: (root_key, chain_key) = KDF(root_key, DH_out)
        let hk = Hkdf::<Sha256>::new(Some(&self.root_key), dh_out.as_bytes());
        let mut output = [0u8; 64];
        hk.expand(b"Signal_RootKey", &mut output)
            .map_err(|e| anyhow::anyhow!("Root key HKDF failed: {}", e))?;
        
        self.root_key.copy_from_slice(&output[0..32]);
        self.chain_key_send = Some(output[32..64].try_into().unwrap());
        
        debug!("Root key and sending chain key updated");
        Ok(())
    }
    
    /// Update receiving chain key when new ratchet key received
    pub fn update_root_and_chain_keys_recv(&mut self, new_ratchet_key: X25519PublicKey) -> Result<()> {
        self.remote_ratchet_key = Some(new_ratchet_key);
        
        let ratchet_key = self.ratchet_key_pair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No ratchet key pair"))?;
        
        let dh_out = ratchet_key.private_key.diffie_hellman(&new_ratchet_key);
        
        // Root key ratchet: (root_key, chain_key) = KDF(root_key, DH_out)
        let hk = Hkdf::<Sha256>::new(Some(&self.root_key), dh_out.as_bytes());
        let mut output = [0u8; 64];
        hk.expand(b"Signal_RootKey", &mut output)
            .map_err(|e| anyhow::anyhow!("Root key HKDF failed: {}", e))?;
        
        self.root_key.copy_from_slice(&output[0..32]);
        self.chain_key_recv = Some(output[32..64].try_into().unwrap());
        self.recv_counter = 0;
        
        debug!("Root key and receiving chain key updated");
        Ok(())
    }
    
    /// Generate message keys for encryption
    pub fn get_message_keys_send(&mut self) -> Result<MessageKeys> {
        let chain_key = self.chain_key_send
            .ok_or_else(|| anyhow::anyhow!("No sending chain key"))?;
        
        let message_keys = MessageKeys::from_chain_key(&chain_key, self.send_counter)?;
        
        // Advance chain key
        self.chain_key_send = Some(Self::advance_chain_key(&chain_key)?);
        self.send_counter += 1;
        
        Ok(message_keys)
    }
    
    /// Generate message keys for decryption
    pub fn get_message_keys_recv(&mut self) -> Result<MessageKeys> {
        let chain_key = self.chain_key_recv
            .ok_or_else(|| anyhow::anyhow!("No receiving chain key"))?;
        
        let message_keys = MessageKeys::from_chain_key(&chain_key, self.recv_counter)?;
        
        // Advance chain key
        self.chain_key_recv = Some(Self::advance_chain_key(&chain_key)?);
        self.recv_counter += 1;
        
        Ok(message_keys)
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
}

/// Message keys for encryption/decryption
#[derive(Debug)]
pub struct MessageKeys {
    pub cipher_key: [u8; 32],
    pub mac_key: [u8; 32],
    pub iv: [u8; 16],
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
        hk.expand(b"Signal_MessageKey_Cipher", &mut cipher_key)
            .map_err(|e| anyhow::anyhow!("Cipher key derivation failed: {}", e))?;
        
        let mut mac_key = [0u8; 32];
        hk.expand(b"Signal_MessageKey_MAC", &mut mac_key)
            .map_err(|e| anyhow::anyhow!("MAC key derivation failed: {}", e))?;
        
        let mut iv = [0u8; 16];
        hk.expand(b"Signal_MessageKey_IV", &mut iv)
            .map_err(|e| anyhow::anyhow!("IV derivation failed: {}", e))?;
        
        Ok(Self {
            cipher_key,
            mac_key,
            iv,
        })
    }
}

/// Signal Protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    pub version: u8,
    pub sender_ratchet_key: [u8; 32],
    pub counter: u32,
    pub previous_counter: u32,
    pub ciphertext: Vec<u8>,
    pub mac: [u8; 32],
}

impl SignalMessage {
    pub fn new(
        sender_ratchet_key: &X25519PublicKey,
        counter: u32,
        previous_counter: u32,
        ciphertext: Vec<u8>,
        mac_key: &[u8; 32],
        local_identity: &X25519PublicKey,
        remote_identity: &X25519PublicKey,
    ) -> Result<Self> {
        let version = 4u8; // Protocol version 4
        let sender_ratchet_bytes = sender_ratchet_key.to_bytes();
        
        // Calculate MAC over message
        let mut mac_input = Vec::new();
        mac_input.push(version);
        mac_input.extend_from_slice(&sender_ratchet_bytes);
        mac_input.extend_from_slice(&counter.to_be_bytes());
        mac_input.extend_from_slice(&previous_counter.to_be_bytes());
        mac_input.extend_from_slice(&ciphertext);
        mac_input.extend_from_slice(&local_identity.to_bytes());
        mac_input.extend_from_slice(&remote_identity.to_bytes());
        
        let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(mac_key)
            .map_err(|e| anyhow::anyhow!("MAC creation failed: {}", e))?;
        mac.update(&mac_input);
        
        let mac_result = mac.finalize().into_bytes();
        let mut mac_bytes = [0u8; 32];
        mac_bytes.copy_from_slice(&mac_result[0..32]);
        
        Ok(Self {
            version,
            sender_ratchet_key: sender_ratchet_bytes,
            counter,
            previous_counter,
            ciphertext,
            mac: mac_bytes,
        })
    }
    
    pub fn verify_mac(
        &self,
        mac_key: &[u8; 32],
        local_identity: &X25519PublicKey,
        remote_identity: &X25519PublicKey,
    ) -> Result<bool> {
        let mut mac_input = Vec::new();
        mac_input.push(self.version);
        mac_input.extend_from_slice(&self.sender_ratchet_key);
        mac_input.extend_from_slice(&self.counter.to_be_bytes());
        mac_input.extend_from_slice(&self.previous_counter.to_be_bytes());
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

/// Session store for managing sessions
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn load_session(&self, address: &ProtocolAddress) -> Result<Option<SessionState>>;
    async fn store_session(&self, address: &ProtocolAddress, session: &SessionState) -> Result<()>;
    async fn contains_session(&self, address: &ProtocolAddress) -> Result<bool>;
    async fn delete_session(&self, address: &ProtocolAddress) -> Result<()>;
}

/// In-memory session store implementation
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<ProtocolAddress, SessionState>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl SessionStore for InMemorySessionStore {
    async fn load_session(&self, address: &ProtocolAddress) -> Result<Option<SessionState>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(address).cloned())
    }
    
    async fn store_session(&self, address: &ProtocolAddress, session: &SessionState) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(address.clone(), session.clone());
        debug!("Session stored for {}", address.name);
        Ok(())
    }
    
    async fn contains_session(&self, address: &ProtocolAddress) -> Result<bool> {
        let sessions = self.sessions.read().await;
        Ok(sessions.contains_key(address))
    }
    
    async fn delete_session(&self, address: &ProtocolAddress) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(address);
        debug!("Session deleted for {}", address.name);
        Ok(())
    }
}

/// Identity key store for managing identity keys
#[async_trait::async_trait]
pub trait IdentityKeyStore: Send + Sync {
    async fn get_identity_key_pair(&self) -> Result<IdentityKeyPair>;
    async fn get_local_registration_id(&self) -> Result<u32>;
    async fn save_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> Result<()>;
    async fn is_trusted_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> Result<bool>;
    async fn get_identity(&self, address: &ProtocolAddress) -> Result<Option<X25519PublicKey>>;
}

/// In-memory identity store implementation
#[derive(Debug)]
pub struct InMemoryIdentityStore {
    identity_key_pair: IdentityKeyPair,
    registration_id: u32,
    trusted_keys: Arc<RwLock<HashMap<ProtocolAddress, X25519PublicKey>>>,
}

impl InMemoryIdentityStore {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let identity_key_pair = IdentityKeyPair::generate();
        let registration_id = rng.next_u32();
        
        Self {
            identity_key_pair,
            registration_id,
            trusted_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl IdentityKeyStore for InMemoryIdentityStore {
    async fn get_identity_key_pair(&self) -> Result<IdentityKeyPair> {
        Ok(self.identity_key_pair.clone())
    }
    
    async fn get_local_registration_id(&self) -> Result<u32> {
        Ok(self.registration_id)
    }
    
    async fn save_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> Result<()> {
        let mut trusted_keys = self.trusted_keys.write().await;
        trusted_keys.insert(address.clone(), *identity_key);
        debug!("Identity saved for {}", address.name);
        Ok(())
    }
    
    async fn is_trusted_identity(&self, address: &ProtocolAddress, identity_key: &X25519PublicKey) -> Result<bool> {
        let trusted_keys = self.trusted_keys.read().await;
        match trusted_keys.get(address) {
            Some(trusted_key) => Ok(trusted_key == identity_key),
            None => Ok(true), // Trust on first use
        }
    }
    
    async fn get_identity(&self, address: &ProtocolAddress) -> Result<Option<X25519PublicKey>> {
        let trusted_keys = self.trusted_keys.read().await;
        Ok(trusted_keys.get(address).copied())
    }
}

/// Main Signal Protocol implementation
pub struct SignalProtocol {
    session_store: Box<dyn SessionStore>,
    identity_store: Box<dyn IdentityKeyStore>,
}

impl SignalProtocol {
    pub fn new(
        session_store: Box<dyn SessionStore>,
        identity_store: Box<dyn IdentityKeyStore>,
    ) -> Self {
        Self {
            session_store,
            identity_store,
        }
    }
    
    /// Create new protocol with in-memory stores
    pub fn new_in_memory() -> Self {
        let session_store = Box::new(InMemorySessionStore::new());
        let identity_store = Box::new(InMemoryIdentityStore::new());
        
        Self::new(session_store, identity_store)
    }
    
    /// Encrypt message using Signal Protocol
    pub async fn encrypt_message(
        &self,
        recipient: &ProtocolAddress,
        plaintext: &[u8],
    ) -> Result<SignalMessage> {
        info!("Encrypting message for {}", recipient.name);
        
        // Load or create session
        let mut session = match self.session_store.load_session(recipient).await? {
            Some(session) => session,
            None => {
                error!("No session found for {}", recipient.name);
                return Err(anyhow::anyhow!("No session established with {}", recipient.name).into());
            }
        };
        
        // Get message keys
        let message_keys = session.get_message_keys_send()?;
        
        // Encrypt message with AES-256-GCM
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&message_keys.cipher_key));
        let nonce = Nonce::from_slice(&message_keys.iv[0..12]); // AES-GCM uses 12-byte nonce
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Get local and remote identity keys
        let local_identity = self.identity_store.get_identity_key_pair().await?;
        let remote_identity = session.remote_identity_key
            .ok_or_else(|| anyhow::anyhow!("No remote identity key"))?;
        
        // Create Signal message
        let sender_ratchet_key = session.ratchet_key_pair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No sender ratchet key"))?
            .public_key;
        
        let signal_message = SignalMessage::new(
            &sender_ratchet_key,
            session.send_counter,
            session.previous_send_counter,
            ciphertext,
            &message_keys.mac_key,
            &local_identity.public_key,
            &remote_identity,
        )?;
        
        // Store updated session
        self.session_store.store_session(recipient, &session).await?;
        
        info!("Message encrypted successfully for {}", recipient.name);
        Ok(signal_message)
    }
    
    /// Decrypt message using Signal Protocol
    pub async fn decrypt_message(
        &self,
        sender: &ProtocolAddress,
        signal_message: &SignalMessage,
    ) -> Result<Vec<u8>> {
        info!("Decrypting message from {}", sender.name);
        
        // Load session
        let mut session = self.session_store.load_session(sender).await?
            .ok_or_else(|| anyhow::anyhow!("No session found for {}", sender.name))?;
        
        // Check if we need to update receiving ratchet
        let sender_ratchet_key = X25519PublicKey::from(signal_message.sender_ratchet_key);
        if session.remote_ratchet_key.is_none() || 
           session.remote_ratchet_key.unwrap() != sender_ratchet_key {
            session.update_root_and_chain_keys_recv(sender_ratchet_key)?;
        }
        
        // Get message keys
        let message_keys = session.get_message_keys_recv()?;
        
        // Verify MAC
        let local_identity = self.identity_store.get_identity_key_pair().await?;
        let remote_identity = session.remote_identity_key
            .ok_or_else(|| anyhow::anyhow!("No remote identity key"))?;
        
        if !signal_message.verify_mac(
            &message_keys.mac_key,
            &local_identity.public_key,
            &remote_identity,
        )? {
            return Err(anyhow::anyhow!("MAC verification failed").into());
        }
        
        // Decrypt message
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&message_keys.cipher_key));
        let nonce = Nonce::from_slice(&message_keys.iv[0..12]);
        
        let plaintext = cipher.decrypt(nonce, signal_message.ciphertext.as_slice())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        // Store updated session
        self.session_store.store_session(sender, &session).await?;
        
        info!("Message decrypted successfully from {}", sender.name);
        Ok(plaintext)
    }
    
    /// Initialize session with remote party (simplified version)
    pub async fn initialize_session(
        &self,
        remote_address: &ProtocolAddress,
        remote_identity_key: X25519PublicKey,
        remote_ephemeral_key: X25519PublicKey,
    ) -> Result<()> {
        info!("Initializing session with {}", remote_address.name);
        
        let local_identity = self.identity_store.get_identity_key_pair().await?;
        let local_ephemeral = IdentityKeyPair::generate();
        
        let mut session = SessionState::new(local_identity.public_key);
        
        // Initialize as Alice (for simplicity - in real implementation, this would be determined by protocol flow)
        session.initialize_alice(
            remote_identity_key,
            remote_ephemeral_key,
            &local_identity,
            &local_ephemeral,
        )?;
        
        // Save identity and session
        self.identity_store.save_identity(remote_address, &remote_identity_key).await?;
        self.session_store.store_session(remote_address, &session).await?;
        
        info!("Session initialized successfully with {}", remote_address.name);
        Ok(())
    }
    
    /// Get our identity key for sharing with others
    pub async fn get_public_identity_key(&self) -> Result<X25519PublicKey> {
        let identity = self.identity_store.get_identity_key_pair().await?;
        Ok(identity.public_key)
    }
    
    /// Get registration ID
    pub async fn get_registration_id(&self) -> Result<u32> {
        self.identity_store.get_local_registration_id().await
    }
}
