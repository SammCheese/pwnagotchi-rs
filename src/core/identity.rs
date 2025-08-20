use std::path;
use crypto::{
    digest::Digest,
};
use rsa::{pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPublicKey}, rand_core::OsRng, traits::SignatureScheme, Pss, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine};
use tokio::process::Command;

use crate::core::log::LOGGER;

const DEFAULT_PATH: &str = "/etc/pwnagotchi/";

pub struct Identity {
  pub path: String,
  pub priv_path: String,
  pub priv_key: Option<RsaPrivateKey>,
  pub pub_path: String,
  pub pub_key: Option<RsaPublicKey>,
  pub fingerprint_path: String,

  pubkey_pem_b64: Option<String>,
  fingerprint: Option<String>,
}

impl Default for Identity {
  fn default() -> Self {
    let path = DEFAULT_PATH.to_string();
    let priv_path = format!("{}id_rsa", path);
    let pub_path = format!("{}.pub", priv_path);
    let fingerprint_path = format!("{}fingerprint", path);
    let priv_key = None;
    let pub_key = None;

    Identity {
      path,
      priv_path,
      priv_key,
      pub_path,
      pub_key,
      fingerprint_path,
      pubkey_pem_b64: None,
      fingerprint: None,
    }
  }
}

impl Identity {
  pub async fn new(path: &str) -> Self {
    let path = path.to_string();
    let priv_path = format!("{}id_rsa", path);
    let pub_path = format!("{}.pub", priv_path);
    let fingerprint_path = format!("{}fingerprint", path);

    let mut ident = Identity {
      path,
      priv_path,
      priv_key: None,
      pub_path,
      pub_key: None,
      fingerprint_path,
      pubkey_pem_b64: None,
      fingerprint: None,
    };
    ident.initialize().await;
    ident
  }

  pub async fn initialize(&mut self) {
    if !path::Path::new(&self.path).exists() {
      std::fs::create_dir_all(&self.path).expect("Failed to create identity directory");
    }

    loop {
      match self.try_load_keys() {
        Ok(_) => break,
        Err(e) => {
          LOGGER.log_error("IDENTITY", &format!("Key load failed: {}. Regenerating...", e));
          let _ = Command::new("pwngrid")
              .arg("-generate")
              .arg("-keys")
              .arg(&self.path)
              .status()
              .await;
        }
      }
    }
  }

  fn try_load_keys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
   let priv_key = RsaPrivateKey::read_pkcs1_pem_file(&self.priv_path)?;
    let pub_key = RsaPublicKey::read_pkcs1_pem_file(&self.pub_path)?;

    self.priv_key = Some(priv_key);
    self.pub_key = Some(pub_key.clone());

    let pem = pub_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)?;
    let pem_bytes = pem.as_bytes();

    self.pubkey_pem_b64 = Some(general_purpose::STANDARD.encode(pem_bytes));

    let hash = Sha256::digest(pem_bytes);
    self.fingerprint = Some(hex::encode(hash));

    std::fs::write(&self.fingerprint_path, self.fingerprint.as_ref().unwrap())?;

    Ok(())
  }

  pub fn sign(&self, message: &str) -> Result<String, String> {
    let key = self.priv_key.as_ref().ok_or("Private key not loaded")?;

    let pss = Pss::new_with_salt::<Sha256>(16);

    let signature = pss.sign(Some(&mut OsRng), key, message.as_bytes())
        .map_err(|e| format!("Signing failed: {}", e))?;

    let signature_b64 = general_purpose::STANDARD.encode(&signature);
    Ok(signature_b64)
  }
}