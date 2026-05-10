use crate::vault::crypto::{derive_key, CryptoKey};

pub trait AuthProvider: Send + Sync {
    /// Touch ID / Windows Hello 등 생체인증 시도. 성공 시 true.
    fn biometric_auth(&self, reason: &str) -> Result<bool, String>;
    /// 암호화 키를 OS 키체인에 저장한다.
    fn store_key(&self, key: &[u8]) -> Result<(), String>;
    /// OS 키체인에서 암호화 키를 불러온다.
    fn load_key(&self) -> Result<Vec<u8>, String>;
}

/// 비밀번호로부터 CryptoKey를 파생한다.
pub fn key_from_password(password: &str) -> CryptoKey {
    let salt = b"shelf-vault-salt-v1";
    let key_bytes = derive_key(password, salt);
    CryptoKey::from_bytes(&key_bytes)
}

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::MacOSAuthProvider as PlatformAuthProvider;
