use std::path;
use crypto::{
    digest::Digest,
    signature::{Keypair, Signer, Verifier},
};
use rsa::{pkcs1::DecodeRsaPrivateKey, rand_core::OsRng, traits::SignatureScheme, Pss, RsaPrivateKey};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine};

const DEFAULT_PATH: &str = "/etc/pwnagotchi/";

pub struct Identity {
  pub path: String,
  pub priv_path: String,
  pub priv_key: String,
  pub pub_path: String,
  pub pub_key: String,
  pub fingerprint_path: String,
}

impl Default for Identity {
  fn default() -> Self {
    let path = DEFAULT_PATH.to_string();
    let priv_path = format!("{}id_rsa", path);
    let pub_path = format!("{}.pub", priv_path);
    let fingerprint_path = format!("{}fingerprint", path);

    Identity {
      path,
      priv_path,
      priv_key: "id_rsa".into(),
      pub_path,
      pub_key: "id_rsa.pub".into(),
      fingerprint_path,
    }
  }
}

impl Identity {
  pub fn new(path: &str) -> Self {
    let path = path.to_string();
    let priv_path = format!("{}id_rsa", path);
    let pub_path = format!("{}.pub", priv_path);
    let fingerprint_path = format!("{}fingerprint", path);

    let mut ident = Identity {
      path,
      priv_path,
      priv_key: "id_rsa".into(),
      pub_path,
      pub_key: "id_rsa.pub".into(),
      fingerprint_path,
    };
    ident.initialize();
    ident
  }

  pub fn sign(&self, message: &str) -> Result<String, String> {
    let key = RsaPrivateKey::from_pkcs1_pem(&self.priv_key)
        .map_err(|e| format!("Key parse failed: {}", e))?;

    let pss = Pss::new_with_salt::<Sha256>(16);

    let signature = pss.sign(Some(&mut OsRng), &key, message.as_bytes())
        .map_err(|e| format!("Signing failed: {}", e))?;

    let signature_b64 = general_purpose::STANDARD.encode(&signature);
    Ok(signature_b64)
  }

  fn initialize(&mut self) {
    if !std::path::Path::new(&self.path).exists() {
      std::fs::create_dir_all(&self.path).expect("Failed to create identity directory");
    }
  }
}