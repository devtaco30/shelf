use crate::db::vault::{self, VaultItem};
use crate::vault::auth::{AuthProvider, PlatformAuthProvider};
use crate::vault::crypto::{decrypt, derive_key, encrypt, CryptoKey};
use once_cell::sync::OnceCell;
use rand::RngCore;
use std::sync::Mutex;
use zeroize::Zeroizing;

// 세션 동안 메모리에만 키를 보관한다. Zeroizing<Vec<u8>>은 drop 시 자동으로 0으로 채워진다.
static SESSION_KEY: OnceCell<Mutex<Option<Zeroizing<Vec<u8>>>>> = OnceCell::new();

fn session_key_store() -> &'static Mutex<Option<Zeroizing<Vec<u8>>>> {
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

/// 비밀번호 + salt로 32바이트 키를 파생한다. Zeroizing으로 래핑해 drop 시 메모리를 소거한다.
fn derive_raw_key(password: &str, salt: &[u8]) -> Zeroizing<Vec<u8>> {
    Zeroizing::new(derive_key(password, salt).to_vec())
}

fn make_crypto_key(bytes: &Zeroizing<Vec<u8>>) -> Result<CryptoKey, String> {
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| "키 크기 오류".to_string())?;
    Ok(CryptoKey::from_bytes(&arr))
}

/// 레거시 고정 salt (v1 이전 vault에서 사용됨)
const LEGACY_SALT: &[u8] = b"shelf-vault-salt-v1";

/// 레거시 vault 마이그레이션: 고정 salt → 랜덤 salt, 전체 아이템 재암호화.
/// 비밀번호 잠금 해제 성공 후 한 번만 실행된다.
fn migrate_to_random_salt(
    password: &str,
    old_key: &CryptoKey,
) -> Result<Zeroizing<Vec<u8>>, String> {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let new_key_bytes = derive_raw_key(password, &salt);
    let new_key = make_crypto_key(&new_key_bytes)?;

    // 모든 아이템을 old_key로 복호화 → new_key로 재암호화
    let items = vault::get_all().map_err(|e| e.to_string())?;
    for item in &items {
        let old_encrypted = vault::get_content(item.id).map_err(|e| e.to_string())?;
        let plaintext = decrypt(old_key, &old_encrypted)?;
        let new_encrypted = encrypt(&new_key, &plaintext)?;
        vault::update_content(item.id, &new_encrypted).map_err(|e| e.to_string())?;
    }

    // 새 salt와 새 키를 Keychain에 저장
    let provider = PlatformAuthProvider;
    provider.store_salt(&salt)?;
    provider.store_key(&new_key_bytes)?;

    Ok(new_key_bytes)
}

/// Vault 최초 설정: 랜덤 salt 생성 → 키 파생 → Keychain에 salt + 키 저장.
#[tauri::command]
pub fn setup_vault(password: String) -> Result<(), String> {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let key_bytes = derive_raw_key(&password, &salt);

    let provider = PlatformAuthProvider;
    provider.store_salt(&salt)?;
    provider.store_key(&key_bytes)?;

    let mut guard = session_key_store().lock().unwrap();
    *guard = Some(key_bytes);
    Ok(())
}

/// Touch ID 또는 비밀번호로 Vault를 잠금 해제한다.
/// password가 None이면 Touch ID로 인증하고 Keychain에서 키를 로드한다.
/// 비밀번호 경로에서 레거시 vault(고정 salt)가 감지되면 자동 마이그레이션을 실행한다.
#[tauri::command]
pub fn unlock_vault(password: Option<String>) -> Result<(), String> {
    let provider = PlatformAuthProvider;

    if password.is_none() {
        // Touch ID 경로: Keychain에서 키를 직접 로드 (salt 불필요)
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
        *guard = Some(Zeroizing::new(key_bytes));
        return Ok(());
    }

    let pw = password.unwrap();

    // Keychain에서 salt 로드. 없으면 레거시 vault로 판단.
    let stored_salt = provider.load_salt();
    let is_legacy = stored_salt.is_err();
    let salt = stored_salt.unwrap_or_else(|_| LEGACY_SALT.to_vec());

    let key_bytes = derive_raw_key(&pw, &salt);
    let key = make_crypto_key(&key_bytes)?;

    // 비밀번호 검증: 아이템이 있으면 복호화 테스트
    if let Ok(items) = vault::get_all() {
        if let Some(item) = items.first() {
            let content = vault::get_content(item.id).map_err(|e| e.to_string())?;
            decrypt(&key, &content)?; // 틀린 비밀번호 → 여기서 에러 반환
        }
    }

    // 레거시 vault: 자동 마이그레이션 (새 랜덤 salt + 전체 재암호화)
    let final_key = if is_legacy {
        migrate_to_random_salt(&pw, &key)?
    } else {
        // 정상 경로: Keychain 키 갱신 (unrestricted ACL 마이그레이션)
        let _ = provider.store_key(&key_bytes);
        key_bytes
    };

    let mut guard = session_key_store().lock().unwrap();
    *guard = Some(final_key);
    Ok(())
}

/// Vault를 잠근다. 세션 키가 drop되면서 메모리의 키 바이트가 0으로 소거된다.
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

#[tauri::command(rename_all = "snake_case")]
pub fn create_vault_item(title: String, content: String, item_type: String) -> Result<i64, String> {
    let key = get_session_key()?;
    let encrypted = encrypt(&key, content.as_bytes())?;
    vault::create(&title, &item_type, &encrypted).map_err(|e| e.to_string())
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
