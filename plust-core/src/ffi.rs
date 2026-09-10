use std::ffi::{c_char, CStr, CString};
use std::ptr::null_mut;

use crate::manifest::Manifest;
use crate::player::Player;
use crate::{fmp4, segment};

#[repr(C)]
pub struct FFISample {
    pub data_ptr: *const u8,
    pub data_len: usize,
    pub pts: f64,
    pub dts: f64,
    pub duration: f64,
    pub is_key: bool,
    pub is_last: bool,
}

#[repr(C)]
pub struct FFICodecConfig {
    pub sps_ptr: *const u8,
    pub sps_len: usize,
    pub pps_ptr: *const u8,
    pub pps_len: usize,
    pub width: u32,
    pub height: u32,
    pub nal_length_size: u8, // Usually 4
}

fn cstr_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s).to_str().ok() }
}

#[no_mangle]
pub extern "C" fn player_new(base_url: *const c_char) -> *mut Player {
    let Some(url) = cstr_to_str(base_url) else {
        return null_mut();
    };
    match Player::new(url) {
        Some(player) => Box::into_raw(Box::new(player)),
        None => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn player_free(player: *mut Player) {
    if player.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(player);
    }
}

#[no_mangle]
pub extern "C" fn player_start_loading(player: *mut Player) {
    if player.is_null() {
        return;
    }
    unsafe {
        (*player).start_loading();
    }
}

#[no_mangle]
pub extern "C" fn player_play(player: *mut Player, now: f64) {
    if player.is_null() {
        return;
    }
    unsafe {
        (*player).play(now);
    }
}

#[no_mangle]
pub extern "C" fn player_pause(player: *mut Player, now: f64) {
    if player.is_null() {
        return;
    }
    unsafe {
        (*player).pause(now);
    }
}

#[no_mangle]
pub extern "C" fn player_seek(player: *mut Player, media: f64, now: f64) {
    if player.is_null() {
        return;
    }
    unsafe {
        (*player).seek(media, now);
    }
}

#[no_mangle]
pub extern "C" fn player_set_playback_rate(player: *mut Player, rate: f64, now: f64) {
    if player.is_null() {
        return;
    }
    unsafe {
        (*player).set_playback_rate(rate, now);
    }
}

#[no_mangle]
pub extern "C" fn player_is_playing(player: *const Player) -> bool {
    if player.is_null() {
        return false;
    }
    unsafe { (*player).is_playing() }
}

#[no_mangle]
pub extern "C" fn player_get_media_time(player: *const Player, now: f64) -> f64 {
    if player.is_null() {
        return 0.0;
    }
    unsafe { (*player).get_media_time(now) }
}

#[no_mangle]
pub extern "C" fn player_pop_sample(player: *mut Player) -> *mut FFISample {
    if player.is_null() {
        return null_mut();
    }
    unsafe {
        match (*player).pop_next_sample() {
            Some(sample) => {
                let data = sample.data.into_boxed_slice();
                let data_len = data.len();
                let data_ptr = Box::into_raw(data) as *const u8;
                Box::into_raw(Box::new(FFISample {
                    data_ptr,
                    data_len,
                    pts: sample.pts,
                    dts: sample.dts,
                    duration: sample.duration,
                    is_key: sample.is_key,
                    is_last: sample.is_last,
                }))
            }
            None => null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn sample_free(sample: *mut FFISample) {
    if sample.is_null() {
        return;
    }
    unsafe {
        let s = Box::from_raw(sample);
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            s.data_ptr as *mut u8,
            s.data_len,
        ));
    }
}

#[no_mangle]
pub extern "C" fn get_manifest_json(base_url: *const c_char) -> *const c_char {
    let Some(url) = cstr_to_str(base_url) else {
        return null_mut();
    };
    let Ok(man) = Manifest::new(url) else {
        return null_mut();
    };

    let Ok(json) = man.to_json() else {
        return null_mut();
    };

    let Ok(c_str) = CString::new(json) else {
        return null_mut();
    };
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(string: *const c_char) {
    if string.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(string as *mut c_char);
    }
}

#[no_mangle]
pub extern "C" fn get_codec_config(base_url: *const c_char) -> *mut FFICodecConfig {
    let Some(url) = cstr_to_str(base_url) else {
        return null_mut();
    };
    let Ok(man) = Manifest::new(url) else {
        return null_mut();
    };

    let init_path = man.init_path(&man.representations[0]);
    let Ok(init_data) = segment::get_segment(&init_path) else {
        return null_mut();
    };

    let Some(config) = fmp4::get_codec_config(&init_data) else {
        return null_mut();
    };

    let sps = config.sps.into_boxed_slice();
    let sps_len = sps.len();
    let sps_ptr = Box::into_raw(sps) as *const u8;

    let pps = config.pps.into_boxed_slice();
    let pps_len = pps.len();
    let pps_ptr = Box::into_raw(pps) as *const u8;

    let ffi_config = FFICodecConfig {
        sps_ptr,
        sps_len,
        pps_ptr,
        pps_len,
        width: config.width,
        height: config.height,
        nal_length_size: config.nal_length_size,
    };

    Box::into_raw(Box::new(ffi_config))
}

#[no_mangle]
pub extern "C" fn free_codec_config(config: *mut FFICodecConfig) {
    if config.is_null() {
        return;
    }
    unsafe {
        let c = Box::from_raw(config);
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            c.sps_ptr as *mut u8,
            c.sps_len,
        ));
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            c.pps_ptr as *mut u8,
            c.pps_len,
        ));
    }
}

#[no_mangle]
pub extern "C" fn register_log_callback(callback: extern "C" fn(*const c_char)) {
    crate::logger::register_log_callback(callback);
}
