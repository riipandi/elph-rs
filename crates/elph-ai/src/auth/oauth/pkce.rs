use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

pub async fn generate_pkce() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = base64_url_no_pad(&bytes);
    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = base64_url_no_pad(&hash);
    (verifier, challenge)
}

fn base64_url_no_pad(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
