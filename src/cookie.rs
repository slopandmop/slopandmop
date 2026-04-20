// HS256 signed session cookie. Factory mints; sm-agent verifies.
//
// Format: base64url(payload).base64url(hmac_sha256(key, payload))
// where payload is JSON {"sm_id":"…","exp":<unix-seconds>}.
//
// The key is unique per sm — factory mints fresh bytes on /sm/provision
// and embeds them in the sm's ee-config. Only factory + that one sm know it.

use anyhow::{anyhow, bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub sm_id: String,
    pub exp: i64,
}

pub fn sign(key: &[u8], payload: &Payload) -> Result<String> {
    let body = serde_json::to_vec(payload)?;
    let body_b64 = URL_SAFE_NO_PAD.encode(&body);
    let mut mac = HmacSha256::new_from_slice(key).map_err(|e| anyhow!("hmac key: {e}"))?;
    mac.update(body_b64.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{body_b64}.{sig_b64}"))
}

pub fn verify(key: &[u8], cookie: &str) -> Result<Payload> {
    let (body_b64, sig_b64) = cookie
        .split_once('.')
        .ok_or_else(|| anyhow!("cookie: no separator"))?;
    let mut mac = HmacSha256::new_from_slice(key).map_err(|e| anyhow!("hmac key: {e}"))?;
    mac.update(body_b64.as_bytes());
    let expected = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    if expected.as_bytes() != sig_b64.as_bytes() {
        bail!("cookie: signature mismatch");
    }
    let body = URL_SAFE_NO_PAD.decode(body_b64)?;
    let payload: Payload = serde_json::from_slice(&body)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    if payload.exp < now {
        bail!("cookie: expired at {}, now {now}", payload.exp);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = b"super-secret-key-at-least-32-bytes-long!!";
        let payload = Payload {
            sm_id: "abc123".into(),
            exp: i64::MAX,
        };
        let signed = sign(key, &payload).unwrap();
        let recovered = verify(key, &signed).unwrap();
        assert_eq!(recovered.sm_id, "abc123");
    }

    #[test]
    fn tampered_fails() {
        let key = b"super-secret-key-at-least-32-bytes-long!!";
        let signed = sign(
            key,
            &Payload {
                sm_id: "abc".into(),
                exp: i64::MAX,
            },
        )
        .unwrap();
        let (body, _) = signed.split_once('.').unwrap();
        let tampered = format!("{body}.AAAA");
        assert!(verify(key, &tampered).is_err());
    }

    #[test]
    fn expired_fails() {
        let key = b"k";
        let signed = sign(
            key,
            &Payload {
                sm_id: "abc".into(),
                exp: 0,
            },
        )
        .unwrap();
        assert!(verify(key, &signed).is_err());
    }
}
