pub trait AuthProvider: Send + Sync {
    /// Touch ID / Windows Hello 등 생체인증 시도. 성공 시 true.
    fn biometric_auth(&self, reason: &str) -> Result<bool, String>;
    /// 암호화 키를 OS 키체인에 저장한다.
    fn store_key(&self, key: &[u8]) -> Result<(), String>;
    /// OS 키체인에서 암호화 키를 불러온다.
    fn load_key(&self) -> Result<Vec<u8>, String>;
    /// PBKDF2 salt를 OS 키체인에 저장한다.
    fn store_salt(&self, salt: &[u8]) -> Result<(), String>;
    /// OS 키체인에서 PBKDF2 salt를 불러온다.
    fn load_salt(&self) -> Result<Vec<u8>, String>;
}

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::MacOSAuthProvider as PlatformAuthProvider;
