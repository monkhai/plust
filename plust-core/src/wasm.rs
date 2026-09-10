use crate::manifest::Manifest;
use crate::player::Player;
use crate::{fmp4, segment};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

// Global storage for Player instances
// JavaScript gets an ID (number), Rust keeps the actual Player
static PLAYERS: Mutex<Option<HashMap<u32, Player>>> = Mutex::new(None);
static NEXT_PLAYER_ID: Mutex<u32> = Mutex::new(1);

fn get_players() -> std::sync::MutexGuard<'static, Option<HashMap<u32, Player>>> {
    PLAYERS.lock().unwrap()
}

fn init_players() {
    let mut players = get_players();
    if players.is_none() {
        *players = Some(HashMap::new());
    }
}

// ============================================================================
// MANIFEST
// ============================================================================

#[wasm_bindgen]
pub async fn get_manifest_json(base_url: &str) -> Result<String, JsValue> {
    let manifest = Manifest::new(base_url)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    manifest
        .to_json()
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ============================================================================
// CODEC CONFIG
// ============================================================================

#[wasm_bindgen]
pub struct CodecConfig {
    sps: Vec<u8>,
    pps: Vec<u8>,
    width: u32,
    height: u32,
    nal_length_size: u8,
    avcc_data: Vec<u8>,
}

#[wasm_bindgen]
impl CodecConfig {
    #[wasm_bindgen(getter)]
    pub fn sps(&self) -> Vec<u8> {
        self.sps.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pps(&self) -> Vec<u8> {
        self.pps.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter)]
    pub fn nal_length_size(&self) -> u8 {
        self.nal_length_size
    }

    #[wasm_bindgen(getter)]
    pub fn avcc_data(&self) -> Vec<u8> {
        self.avcc_data.clone()
    }
}

#[wasm_bindgen]
pub async fn get_codec_config(base_url: &str) -> Result<CodecConfig, JsValue> {
    let manifest = Manifest::new(base_url)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let init_path = manifest.init_path(&manifest.representations[0]);
    let init_data = segment::get_segment(&init_path)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let config = fmp4::get_codec_config(&init_data)
        .ok_or_else(|| JsValue::from_str("Failed to get codec config"))?;

    Ok(CodecConfig {
        sps: config.sps,
        pps: config.pps,
        width: config.width,
        height: config.height,
        nal_length_size: config.nal_length_size,
        avcc_data: config.avcc_data,
    })
}

// ============================================================================
// SAMPLE
// ============================================================================

#[wasm_bindgen]
pub struct Sample {
    data: Vec<u8>,
    pts: f64,
    dts: f64,
    duration: f64,
    is_key: bool,
    is_last: bool,
}

#[wasm_bindgen]
impl Sample {
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pts(&self) -> f64 {
        self.pts
    }

    #[wasm_bindgen(getter)]
    pub fn dts(&self) -> f64 {
        self.dts
    }

    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> f64 {
        self.duration
    }

    #[wasm_bindgen(getter)]
    pub fn is_key(&self) -> bool {
        self.is_key
    }

    #[wasm_bindgen(getter)]
    pub fn is_last(&self) -> bool {
        self.is_last
    }
}

// ============================================================================
// PLAYER
// ============================================================================

#[wasm_bindgen]
pub async fn player_new(base_url: &str) -> Result<u32, JsValue> {
    init_players();

    let player = Player::new(base_url)
        .await
        .ok_or_else(|| JsValue::from_str("Failed to create player"))?;

    let mut next_id = NEXT_PLAYER_ID.lock().unwrap();
    let player_id = *next_id;
    *next_id += 1;
    drop(next_id);

    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        map.insert(player_id, player);
    }

    Ok(player_id)
}

#[wasm_bindgen]
pub fn player_free(player_id: u32) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        map.remove(&player_id);
    }
}

#[wasm_bindgen]
pub fn player_start_loading(player_id: u32) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            player.start_loading();
        }
    }
}

#[wasm_bindgen]
pub fn player_play(player_id: u32, now: f64) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            player.play(now);
        }
    }
}

#[wasm_bindgen]
pub fn player_pause(player_id: u32, now: f64) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            player.pause(now);
        }
    }
}

#[wasm_bindgen]
pub fn player_seek(player_id: u32, media_time: f64, now: f64) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            player.seek(media_time, now);
        }
    }
}

#[wasm_bindgen]
pub fn player_set_playback_rate(player_id: u32, rate: f64, now: f64) {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            player.set_playback_rate(rate, now);
        }
    }
}

#[wasm_bindgen]
pub fn player_is_playing(player_id: u32) -> bool {
    let players = get_players();
    if let Some(map) = players.as_ref() {
        if let Some(player) = map.get(&player_id) {
            return player.is_playing();
        }
    }
    false
}

#[wasm_bindgen]
pub fn player_get_media_time(player_id: u32, now: f64) -> f64 {
    let players = get_players();
    if let Some(map) = players.as_ref() {
        if let Some(player) = map.get(&player_id) {
            return player.get_media_time(now);
        }
    }
    0.0
}

#[wasm_bindgen]
pub fn player_pop_sample(player_id: u32) -> Option<Sample> {
    let mut players = get_players();
    if let Some(map) = players.as_mut() {
        if let Some(player) = map.get_mut(&player_id) {
            if let Some(sample) = player.pop_next_sample() {
                return Some(Sample {
                    data: sample.data,
                    pts: sample.pts,
                    dts: sample.dts,
                    duration: sample.duration,
                    is_key: sample.is_key,
                    is_last: sample.is_last,
                });
            }
        }
    }
    None
}
