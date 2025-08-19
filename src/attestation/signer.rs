use crate::Result;
use anyhow;
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey, SecretKey, Signer as _};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use blake3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarSignature {
    pub algorithm: String,           // e.g., "ed25519"
    pub signature_b64: String,
    pub public_key_b64: String,
    pub content_hash_hex: String,    // blake3 of file content
    pub context_hash_hex: String,    // app-level context hash
    pub timestamp: String,
    pub attestation_id: Option<String>,
}

pub struct Signer {
    key_dir: PathBuf,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Signer {
    pub fn load_or_generate(key_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&key_dir)?;
        let sk_path = key_dir.join("attest_ed25519.key");
        let pk_path = key_dir.join("attest_ed25519.pub");

        let (signing_key, verifying_key) = if sk_path.exists() && pk_path.exists() {
            let sk_bytes_b64 = fs::read_to_string(&sk_path)?;
            let pk_bytes_b64 = fs::read_to_string(&pk_path)?;
            let sk_bytes = general_purpose::STANDARD.decode(sk_bytes_b64.trim())?;
            let pk_bytes = general_purpose::STANDARD.decode(pk_bytes_b64.trim())?;

            let signing_key = SigningKey::from_bytes(sk_bytes[..32].try_into().map_err(|_| anyhow::anyhow!("bad sk len"))?);
            let verifying_key = VerifyingKey::from_bytes(pk_bytes[..32].try_into().map_err(|_| anyhow::anyhow!("bad pk len"))?)?;
            (signing_key, verifying_key)
        } else {
            let mut rng = OsRng;
            let mut seed = [0u8; 32];
            rng.fill_bytes(&mut seed);
            let secret: SecretKey = seed;
            let signing_key = SigningKey::from_bytes(&secret);
            let verifying_key = signing_key.verifying_key();
            let sk_b64 = general_purpose::STANDARD.encode(signing_key.to_bytes());
            let pk_b64 = general_purpose::STANDARD.encode(verifying_key.to_bytes());
            fs::write(&sk_path, sk_b64)?;
            fs::write(&pk_path, pk_b64)?;
            (signing_key, verifying_key)
        };

        Ok(Self { key_dir, signing_key, verifying_key })
    }

    pub fn sign_bytes(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    pub fn public_key_b64(&self) -> String {
        general_purpose::STANDARD.encode(self.verifying_key.to_bytes())
    }

    pub fn sign_markdown_sidecar(&self, path: &Path, context_hash_hex: &str, attestation_id: Option<String>) -> Result<PathBuf> {
        let content = fs::read(path)?;
        let content_hash_hex = hex::encode(blake3::hash(&content).as_bytes());
        let sig = self.sign_bytes(&content);
        let sig_b64 = general_purpose::STANDARD.encode(sig.to_bytes());

        let sidecar = SidecarSignature {
            algorithm: "ed25519".to_string(),
            signature_b64: sig_b64,
            public_key_b64: self.public_key_b64(),
            content_hash_hex,
            context_hash_hex: context_hash_hex.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            attestation_id,
        };

        let json = serde_json::to_string_pretty(&sidecar)?;
        let sidecar_path = path.with_extension(path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string() + ".sig.json");
        fs::write(&sidecar_path, json)?;
        Ok(sidecar_path)
    }
}
