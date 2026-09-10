use std::ffi::{c_char, CString};

type LogCallback = extern "C" fn(*const c_char);
static mut LOG_CALLBACK: Option<LogCallback> = None;

pub fn register_log_callback(callback: LogCallback) {
    unsafe { LOG_CALLBACK = Some(callback) };
}

pub fn log(msg: &str) {
    let formatted = format!("[Rust] {}", msg);
    if let Ok(c_str) = CString::new(formatted) {
        unsafe {
            if let Some(cb) = LOG_CALLBACK {
                cb(c_str.as_ptr());
            }
        }
    }
}
