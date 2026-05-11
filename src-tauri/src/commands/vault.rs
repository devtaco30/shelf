use crate::db::vault::{self, VaultItem};
use crate::vault::auth::{key_from_password, AuthProvider, PlatformAuthProvider};
use crate::vault::crypto::{decrypt, derive_key, encrypt, CryptoKey};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

// 세션 동안 메모리에만 키를 보관한다. 앱 재시작 또는 잠금 시 None으로 초기화.
static SESSION_KEY: OnceCell<Mutex<Option<Vec<u8>>>> = OnceCell::new();

fn session_key_store() -> &'static Mutex<Option<Vec<u8>>> {
    SESSION_KEY.get_or_init(|| Mutex::new(None))
}

fn get_session_key() -> Result<CryptoKey, String> {
    let guard = session_key_store().lock().unwrap();
    match guard.as_ref() {
        Some(bytes) => {
            let arr: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| "키 크기 오류".to_string())?;
            Ok(CryptoKey::from_bytes(&arr))
        }
        None => Err("Vault가 잠겨있습니다".to_string()),
    }
}

fn derive_raw_key(password: &str) -> Vec<u8> {
    derive_key(password, b"shelf-vault-salt-v1").to_vec()
}

/// Vault 최초 설정: 비밀번호로 키를 생성하고 Keychain에 저장한다.
#[tauri::command]
pub fn setup_vault(password: String) -> Result<(), String> {
    let _ = key_from_password(&password);
    let provider = PlatformAuthProvider;
    let key_bytes = derive_raw_key(&password);
    provider.store_key(&key_bytes)?;
    let mut guard = session_key_store().lock().unwrap();
    *guard = Some(key_bytes);
    Ok(())
}

/// Touch ID 또는 비밀번호로 Vault를 잠금 해제한다.
/// password가 None이면 Touch ID를 시도하고 Keychain에서 키를 로드한다.
#[tauri::command]
pub fn unlock_vault(password: Option<String>) -> Result<(), String> {
    let provider = PlatformAuthProvider;

    if password.is_none() {
        let ok = provider
            .biometric_auth("Shelf Vault 잠금 해제")
            .map_err(|e| format!("[biometric_auth 오류] {e}"))?;
        if !ok {
            return Err("[biometric_auth] success=false 반환됨".to_string());
        }
        let key_bytes = provider
            .load_key()
            .map_err(|e| format!("[load_key 오류] {e}"))?;
        let mut guard = session_key_store().lock().unwrap();
        *guard = Some(key_bytes);
        return Ok(());
    }

    // 비밀번호로 해제
    let pw = password.unwrap();
    let key_bytes = derive_raw_key(&pw);

    // 비밀번호 검증: 아이템이 하나라도 있으면 복호화 테스트
    if let Ok(items) = vault::get_all() {
        if let Some(item) = items.first() {
            let arr: [u8; 32] = key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| "키 오류")?;
            let test_key = CryptoKey::from_bytes(&arr);
            let content = vault::get_content(item.id).map_err(|e| e.to_string())?;
            decrypt(&test_key, &content)?;
        }
    }

    let mut guard = session_key_store().lock().unwrap();
    *guard = Some(key_bytes.clone());
    drop(guard);
    // 비밀번호 인증 성공 시 키체인 아이템을 unrestricted ACL로 재생성 (다이얼로그 제거 마이그레이션)
    let _ = provider.store_key(&key_bytes);
    Ok(())
}

/// Vault를 잠근다 (세션 키 삭제).
#[tauri::command]
pub fn lock_vault() -> Result<(), String> {
    let mut guard = session_key_store().lock().unwrap();
    *guard = None;
    Ok(())
}

#[tauri::command]
pub fn get_vault_items() -> Result<Vec<VaultItem>, String> {
    vault::get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_vault_item(title: String, content: String) -> Result<i64, String> {
    let key = get_session_key()?;
    let encrypted = encrypt(&key, content.as_bytes())?;
    vault::create(&title, &encrypted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vault_content(id: i64) -> Result<String, String> {
    let key = get_session_key()?;
    let encrypted = vault::get_content(id).map_err(|e| e.to_string())?;
    let decrypted = decrypt(&key, &encrypted)?;
    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_vault_item(id: i64, title: String, content: String) -> Result<(), String> {
    let key = get_session_key()?;
    let encrypted = encrypt(&key, content.as_bytes())?;
    vault::update(id, &title, &encrypted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_vault_item(id: i64) -> Result<(), String> {
    vault::delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_vault_unlocked() -> bool {
    session_key_store().lock().unwrap().is_some()
}

#[tauri::command]
pub fn vault_initialized() -> bool {
    PlatformAuthProvider.load_key().is_ok()
}
