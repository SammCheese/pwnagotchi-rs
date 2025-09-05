use std::{
  path,
  process::{Command, exit},
};

use base64::{Engine, engine::general_purpose};
use nix::libc::EXIT_FAILURE;
use rsa::{
  Pss, RsaPrivateKey, RsaPublicKey,
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
  rand_core::OsRng,
  traits::SignatureScheme,
};
use sha2::{Digest, Sha256};

use crate::core::{config::config, log::LOGGER};

pub struct Identity {
  pub path: String,
  priv_path: String,
  priv_key: Option<RsaPrivateKey>,
  pub_path: String,
  pub_key: Option<RsaPublicKey>,
  fingerprint_path: String,

  pubkey_pem_b64: Option<String>,
  pub fingerprint: Option<String>,
}

impl Default for Identity {
  fn default() -> Self {
    let path = config().identity.path.to_string();
    let priv_path = format!("{path}id_rsa");
    let pub_path = format!("{priv_path}.pub");
    let fingerprint_path = format!("{path}fingerprint");
    let priv_key = None;
    let pub_key = None;

    Self {
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
  pub async fn new() -> Self {
    let path = config().identity.path.to_string();
    let priv_path = format!("{path}id_rsa");
    let pub_path = format!("{priv_path}.pub");
    let fingerprint_path = format!("{path}fingerprint");

    let mut ident = Self {
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
    if !path::Path::new(&self.path).exists()
      && let Err(e) = tokio::fs::create_dir_all(&self.path).await
    {
      LOGGER.log_error(
        "IDENTITY",
        &format!("Failed to create identity directory {:?}: {e}", &self.path),
      );

      exit(EXIT_FAILURE);
    }

    loop {
      match self.try_load_keys() {
        Ok(()) => {
          break;
        }
        Err(e) => {
          LOGGER.log_error("IDENTITY", &format!("Key load failed: {e}. Regenerating..."));

          let _ = Command::new("pwngrid").arg("-generate").arg("-keys").arg(&self.path).status();
        }
      }

      if self.priv_key.is_some() && self.pub_key.is_some() {
        break;
      }

      std::thread::sleep(std::time::Duration::from_secs(5));
    }
  }

  pub fn fingerprint(&self) -> &str {
    self.fingerprint.as_deref().unwrap_or("unknown")
  }

  fn try_load_keys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let priv_key = RsaPrivateKey::read_pkcs1_pem_file(&self.priv_path)?;
    let pub_key = RsaPublicKey::from(&priv_key);

    self.priv_key = Some(priv_key);
    self.pub_key = Some(pub_key.clone());

    let pem = pub_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
    let pem_bytes = pem.as_bytes();

    self.pubkey_pem_b64 = Some(general_purpose::STANDARD.encode(pem_bytes));

    let hash = Sha256::digest(pem_bytes);

    self.fingerprint = Some(hex::encode(hash));

    if let Some(fingerprint) = &self.fingerprint {
      std::fs::write(&self.fingerprint_path, fingerprint)?;
    } else {
      return Err("Fingerprint not generated".into());
    }

    Ok(())
  }

  /// Signs a message using the loaded private key and returns the signature as
  /// a base64 string.
  ///
  /// # Errors
  /// Returns an error if the private key is not loaded or if the signing
  /// operation fails.
  pub fn sign(&self, message: &str) -> Result<String, String> {
    let key = self.priv_key.as_ref().ok_or("Private key not loaded")?;

    let pss = Pss::new_with_salt::<Sha256>(16);

    let signature = pss
      .sign(Some(&mut OsRng), key, message.as_bytes())
      .map_err(|e| format!("Signing failed: {e}"))?;

    let signature_b64 = general_purpose::STANDARD.encode(&signature);

    Ok(signature_b64)
  }
}
