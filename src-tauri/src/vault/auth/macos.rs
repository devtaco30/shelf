#![allow(unexpected_cfgs)]

use super::AuthProvider;
use block::ConcreteBlock;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use security_framework::passwords::get_generic_password;
use std::sync::{Arc, Mutex};

const SERVICE: &str = "com.shelf.app";
const ACCOUNT: &str = "vault-key";

pub struct MacOSAuthProvider;

impl AuthProvider for MacOSAuthProvider {
    fn biometric_auth(&self, reason: &str) -> Result<bool, String> {
        authenticate_touch_id(reason)
    }

    fn store_key(&self, key: &[u8]) -> Result<(), String> {
        // SecAccessCreate(NULL trusted list) 방식으로 저장 → load 시 다이얼로그 없음
        unsafe { store_key_unrestricted(key) }
    }

    fn load_key(&self) -> Result<Vec<u8>, String> {
        get_generic_password(SERVICE, ACCOUNT).map_err(|e| e.to_string())
    }
}

/// 키체인 아이템을 "모든 앱 허용, 확인 없음" ACL로 저장한다.
/// SecAccessCreate에 NULL trusted list를 넘기면 어느 앱이든 다이얼로그 없이 접근 가능.
unsafe fn store_key_unrestricted(key: &[u8]) -> Result<(), String> {
    use std::{ffi::c_void, ptr};

    type Cf = *const c_void;

    #[link(name = "Security", kind = "framework")]
    extern "C" {
        static kSecClass: Cf;
        static kSecClassGenericPassword: Cf;
        static kSecAttrService: Cf;
        static kSecAttrAccount: Cf;
        static kSecValueData: Cf;
        static kSecAttrAccess: Cf;

        fn SecAccessCreate(desc: Cf, trusted_list: Cf, out: *mut Cf) -> i32;
        fn SecItemDelete(query: Cf) -> i32;
        fn SecItemAdd(attrs: Cf, result: *mut Cf) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithBytes(
            alloc: Cf,
            bytes: *const u8,
            len: isize,
            encoding: u32,
            is_external: u8,
        ) -> Cf;
        fn CFDataCreate(alloc: Cf, bytes: *const u8, len: isize) -> Cf;
        fn CFDictionaryCreate(
            alloc: Cf,
            keys: *const Cf,
            values: *const Cf,
            num: isize,
            key_cbs: *const c_void,
            val_cbs: *const c_void,
        ) -> Cf;
        fn CFRelease(cf: Cf);
        static kCFTypeDictionaryKeyCallBacks: [u8; 56];
        static kCFTypeDictionaryValueCallBacks: [u8; 56];
    }

    const UTF8_ENCODING: u32 = 0x08000100;

    let make_cfstr = |s: &[u8]| -> Cf {
        CFStringCreateWithBytes(ptr::null(), s.as_ptr(), s.len() as isize, UTF8_ENCODING, 0)
    };

    let cf_service = make_cfstr(SERVICE.as_bytes());
    let cf_account = make_cfstr(ACCOUNT.as_bytes());
    let cf_desc = make_cfstr(b"Shelf Vault Key");

    // NULL trusted_list → 확인 없이 모든 앱 허용
    let mut access: Cf = ptr::null();
    let status = SecAccessCreate(cf_desc, ptr::null(), &mut access);
    CFRelease(cf_desc);
    if status != 0 {
        CFRelease(cf_service);
        CFRelease(cf_account);
        return Err(format!("SecAccessCreate 실패: {status}"));
    }

    let kcb = &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void;
    let vcb = &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void;

    // 기존 아이템 삭제 (없으면 무시)
    {
        let keys = [kSecClass, kSecAttrService, kSecAttrAccount];
        let vals = [kSecClassGenericPassword, cf_service, cf_account];
        let query = CFDictionaryCreate(ptr::null(), keys.as_ptr(), vals.as_ptr(), 3, kcb, vcb);
        SecItemDelete(query);
        CFRelease(query);
    }

    // unrestricted access 포함해서 새로 저장
    let cf_data = CFDataCreate(ptr::null(), key.as_ptr(), key.len() as isize);
    let keys = [kSecClass, kSecAttrService, kSecAttrAccount, kSecValueData, kSecAttrAccess];
    let vals = [kSecClassGenericPassword, cf_service, cf_account, cf_data, access];
    let attrs = CFDictionaryCreate(ptr::null(), keys.as_ptr(), vals.as_ptr(), 5, kcb, vcb);
    let status = SecItemAdd(attrs, ptr::null_mut());

    CFRelease(attrs);
    CFRelease(cf_data);
    CFRelease(cf_service);
    CFRelease(cf_account);
    CFRelease(access);

    if status == 0 {
        Ok(())
    } else {
        Err(format!("SecItemAdd 실패: {status}"))
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
