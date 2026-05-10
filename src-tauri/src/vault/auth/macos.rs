use super::AuthProvider;
use block::ConcreteBlock;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use security_framework::passwords::{get_generic_password, set_generic_password};
use std::sync::{Arc, Mutex};

const SERVICE: &str = "com.shelf.app";
const ACCOUNT: &str = "vault-key";

pub struct MacOSAuthProvider;

impl AuthProvider for MacOSAuthProvider {
    fn biometric_auth(&self, reason: &str) -> Result<bool, String> {
        authenticate_touch_id(reason)
    }

    fn store_key(&self, key: &[u8]) -> Result<(), String> {
        set_generic_password(SERVICE, ACCOUNT, key).map_err(|e| e.to_string())
    }

    fn load_key(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, ACCOUNT).map_err(|e| e.to_string())
    }
}

fn authenticate_touch_id(reason: &str) -> Result<bool, String> {
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_clone = Arc::clone(&tx);

    unsafe {
        let context: *mut Object = msg_send![class!(LAContext), new];
        let mut error_ptr: *mut Object = std::ptr::null_mut();

        // LAPolicyDeviceOwnerAuthenticationWithBiometrics = 1
        let policy: i64 = 1;

        let can_auth: bool =
            msg_send![context, canEvaluatePolicy:policy error:&mut error_ptr];
        if !can_auth {
            let _: () = msg_send![context, release];
            return Err("Touch ID를 사용할 수 없습니다".to_string());
        }

        // 콜백 블록 생성
        let block = ConcreteBlock::new(move |success: bool, _err: *mut Object| {
            if let Some(sender) = tx_clone.lock().unwrap().take() {
                let _ = sender.send(success);
            }
        });
        let block = block.copy();

        // NSString 이유 메시지
        let reason_bytes = reason.as_bytes();
        let ns_reason: *mut Object = {
            let alloc: *mut Object = msg_send![class!(NSString), alloc];
            msg_send![alloc,
                initWithBytes:reason_bytes.as_ptr()
                length:reason_bytes.len()
                encoding:4usize  // NSUTF8StringEncoding
            ]
        };

        let block_ptr = &*block as *const block::Block<_, _> as *const std::ffi::c_void;
        let (): () = msg_send![context,
            evaluatePolicy:policy
            localizedReason:ns_reason
            reply:block_ptr
        ];

        let _: () = msg_send![ns_reason, release];

        // 결과 대기 (최대 60초)
        let result = rx
            .recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or(false);

        let _: () = msg_send![context, release];
        Ok(result)
    }
}
