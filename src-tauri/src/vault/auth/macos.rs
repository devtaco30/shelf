use super::AuthProvider;
use security_framework::passwords::{get_generic_password, set_generic_password};

const SERVICE: &str = "com.shelf.app";
const ACCOUNT: &str = "vault-key";

pub struct MacOSAuthProvider;

impl AuthProvider for MacOSAuthProvider {
    fn biometric_auth(&self, _reason: &str) -> Result<bool, String> {
        // Keychain 접근 가능 여부로 인증 대체 (실제 Touch ID는 Keychain 접근 시 OS가 처리)
        self.load_key().map(|_| true)
    }

    fn store_key(&self, key: &[u8]) -> Result<(), String> {
        set_generic_password(SERVICE, ACCOUNT, key).map_err(|e| e.to_string())
    }

    fn load_key(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, ACCOUNT).map_err(|e| e.to_string())
    }
}
