import wasmInit, {
    get_manifest_json,
    get_codec_config,
    player_new,
    player_free,
    player_start_loading,
    player_play,
    player_pause,
    player_seek,
    player_get_media_time,
    player_pop_sample,
    player_is_playing,
    player_set_playback_rate
} from '../plust-web/pkg/plust_core.js'

/**
 * PlustCore - Static namespace for WASM functions
 * No state, just exposes raw WASM bindings
 */
export class PlustCore {
    static #initialized = false

    /**
     * Initialize the WASM module (call once at startup)
     */
    static async init() {
        if (!this.#initialized) {
            await wasmInit()
            this.#initialized = true
        }
    }

    /**
     * Get current time in seconds (helper)
     */
    static now() {
        return performance.now() / 1000
    }

    // ============================================================================
    // RAW WASM FUNCTIONS - Direct exports
    // ============================================================================

    static getManifestJson = get_manifest_json
    static getCodecConfig = get_codec_config
    static playerNew = player_new
    static playerFree = player_free
    static playerStartLoading = player_start_loading
    static playerPlay = player_play
    static playerPause = player_pause
    static playerSeek = player_seek
    static playerGetMediaTime = player_get_media_time
    static playerPopSample = player_pop_sample
    static playerIsPlaying = player_is_playing
    static playerSetPlaybackRate = player_set_playback_rate
}
