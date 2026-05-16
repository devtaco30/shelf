#![allow(unexpected_cfgs)]

use super::AuthProvider;
use block::ConcreteBlock;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use std::sync::{Arc, Mutex};

const SERVICE: &str = "com.shelf.app";
const KEY_ACCOUNT: &str = "vault-key";
const SALT_ACCOUNT: &str = "vault-salt";
const DB_KEY_ACCOUNT: &str = "db-key";

pub struct MacOSAuthProvider;

impl AuthProvider for MacOSAuthProvider {
    fn biometric_auth(&self, reason: &str) -> Result<bool, String> {
        authenticate_touch_id(reason)
    }

    fn store_key(&self, key: &[u8]) -> Result<(), String> {
        store_secure(KEY_ACCOUNT, key)
    }

    fn load_key(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, KEY_ACCOUNT).map_err(|e| e.to_string())
    }

    fn store_salt(&self, salt: &[u8]) -> Result<(), String> {
        store_secure(SALT_ACCOUNT, salt)
    }

    fn load_salt(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, SALT_ACCOUNT).map_err(|e| e.to_string())
    }

    fn store_db_key(&self, key: &[u8]) -> Result<(), String> {
        store_secure(DB_KEY_ACCOUNT, key)
    }

    fn load_db_key(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, DB_KEY_ACCOUNT).map_err(|e| e.to_string())
    }
}

/// 키체인 아이템을 생성 앱 전용 ACL로 저장한다.
/// SecItemUpdate는 기존 ACL을 유지하므로 삭제 후 재생성해야 ACL이 갱신된다.
fn store_secure(account: &str, data: &[u8]) -> Result<(), String> {
    let _ = delete_generic_password(SERVICE, account); // 기존 아이템(구 ACL 포함) 제거
    set_generic_password(SERVICE, account, data).map_err(|e| e.to_string())
}

fn authenticate_touch_id(reason: &str) -> Result<bool, String> {
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_clone = Arc::clone(&tx);

    unsafe {
        let context: *mut Object = msg_send![class!(LAContext), new];
        let mut error_ptr: *mut Object = std::ptr::null_mut();

        let policy: i64 = 1; // LAPolicyDeviceOwnerAuthenticationWithBiometrics

        let can_auth: bool =
            msg_send![context, canEvaluatePolicy:policy error:&mut error_ptr];
        if !can_auth {
            let _: () = msg_send![context, release];
            return Err("Touch ID를 사용할 수 없습니다".to_string());
        }

        let block = ConcreteBlock::new(move |success: bool, _err: *mut Object| {
            if let Some(sender) = tx_clone.lock().unwrap().take() {
                let _ = sender.send(success);
            }
        });
        let block = block.copy();

        let reason_bytes = reason.as_bytes();
        let ns_reason: *mut Object = {
            let alloc: *mut Object = msg_send![class!(NSString), alloc];
            msg_send![alloc,
                initWithBytes:reason_bytes.as_ptr()
                length:reason_bytes.len()
                encoding:4usize
            ]
        };

        let block_ptr = &*block as *const block::Block<_, _> as *const std::ffi::c_void;
        let (): () = msg_send![context,
            evaluatePolicy:policy
            localizedReason:ns_reason
            reply:block_ptr
        ];

        let _: () = msg_send![ns_reason, release];

        let result = rx
            .recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or(false);

        let _: () = msg_send![context, release];
        Ok(result)
    }
}
