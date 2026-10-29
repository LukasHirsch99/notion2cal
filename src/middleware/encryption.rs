use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Generate, Payload},
};
use secrecy::SecretString;

use crate::config::Keyring;

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

pub fn encrypt_token(
    version: u8,
    key: &[u8; 32],
    token: &str,
    aad: &str,
) -> anyhow::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::generate();
    let ct = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: token.as_bytes(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| anyhow::anyhow!("encrypt failed"))?;

    let mut out = Vec::with_capacity(1 + NONCE_LEN + ct.len());
    out.push(version);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_token(keyring: &Keyring, blob: &[u8], aad: &str) -> anyhow::Result<SecretString> {
    let (&ver, rest) = blob
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty blob"))?;
    anyhow::ensure!(rest.len() >= NONCE_LEN + TAG_LEN, "blob too short");
    let (nonce, ct) = rest.split_at(NONCE_LEN);

    let key = keyring
        .get(ver)
        .ok_or_else(|| anyhow::anyhow!("unknown key version {ver}"))?;
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| anyhow::anyhow!("invalid key length"))?;
    let nonce = Nonce::try_from(nonce).map_err(|_| anyhow::anyhow!("invalid nonce length"))?;
    let pt = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: ct,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| anyhow::anyhow!("decrypt failed"))?;
    Ok(SecretString::from(String::from_utf8(pt)?))
}
