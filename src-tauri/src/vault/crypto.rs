use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

pub struct CryptoKey(Key<Aes256Gcm>);

impl CryptoKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        CryptoKey(*Key::<Aes256Gcm>::from_slice(bytes))
    }
}

/// content를 암호화하고 [nonce(12) || ciphertext] 형태로 반환한다.
pub fn encrypt(key: &CryptoKey, content: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(&key.0);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, content)
        .map_err(|e| e.to_string())?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// [nonce(12) || ciphertext] 형태의 데이터를 복호화한다.
pub fn decrypt(key: &CryptoKey, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("데이터가 너무 짧음".to_string());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(&key.0);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())
}

/// 비밀번호로부터 32바이트 키를 파생한다 (PBKDF2-HMAC-SHA256).
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> CryptoKey {
        let key_bytes = derive_key("test-password", b"test-salt-16byte");
        CryptoKey::from_bytes(&key_bytes)
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello shelf vault";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_differs_from_plaintext() {
        let key = test_key();
        let plaintext = b"secret mnemonic words here";
        let encrypted = encrypt(&key, plaintext).unwrap();
        assert_ne!(encrypted, plaintext.to_vec());
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key1 = test_key();
        let key2_bytes = derive_key("wrong-password", b"test-salt-16byte");
        let key2 = CryptoKey::from_bytes(&key2_bytes);
        let encrypted = encrypt(&key1, b"secret").unwrap();
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_truncated_data_fails() {
        let key = test_key();
        assert!(decrypt(&key, &[0u8; 5]).is_err());
    }

    #[test]
    fn test_derive_key_is_deterministic() {
        let k1 = derive_key("password", b"salt");
        let k2 = derive_key("password", b"salt");
        assert_eq!(k1, k2);
    }
}
