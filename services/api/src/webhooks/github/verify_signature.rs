use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Verifies that the given `signature` (in the form `sha256=...`)
/// matches the HMAC of the `body` using `secret`.
pub fn verify_github_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    if !signature.starts_with("sha256=") {
        return false;
    }
    let sig = &signature["sha256=".len()..];

    let Ok(signature_bytes) = hex::decode(sig) else {
        return false;
    };

    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);

    mac.verify_slice(&signature_bytes).is_ok()
}
