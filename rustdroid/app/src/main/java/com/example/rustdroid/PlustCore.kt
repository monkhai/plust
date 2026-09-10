@file:Suppress("FunctionName")

package com.example.rustdroid

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

@Structure.FieldOrder("data_ptr", "data_len", "pts", "dts", "duration", "is_key")
open class FFISample(p: Pointer? = null) : Structure(p) {
    @JvmField var data_ptr: Pointer? = null
    @JvmField var data_len: Long = 0
    @JvmField var pts: Double = 0.0
    @JvmField var dts: Double = 0.0
    @JvmField var duration: Double = 0.0
    @JvmField var is_key: Byte = 0
}

@Structure.FieldOrder(
        "sps_ptr",
        "sps_len",
        "pps_ptr",
        "pps_len",
        "width",
        "height",
        "nal_length_size"
)
open class FFICodecConfig(p: Pointer? = null) : Structure(p) {
    @JvmField var sps_ptr: Pointer? = null
    @JvmField var sps_len: Long = 0
    @JvmField var pps_ptr: Pointer? = null
    @JvmField var pps_len: Long = 0
    @JvmField var width: Int = 0
    @JvmField var height: Int = 0
    @JvmField var nal_length_size: Byte = 0
}

interface PlustCoreLib : Library {
    companion object {
        val INSTANCE: PlustCoreLib = Native.load("plust_core", PlustCoreLib::class.java)
    }

    fun player_new(base_url: String): Pointer?
    fun player_free(player: Pointer)
    fun player_start_loading(player: Pointer)
    fun player_play(player: Pointer, now: Double)
    fun player_pause(player: Pointer, now: Double)
    fun player_seek(player: Pointer, media: Double, now: Double)
    fun player_set_playback_rate(player: Pointer, rate: Double, now: Double)
    fun player_is_playing(player: Pointer): Boolean
    fun player_get_media_time(player: Pointer, now: Double): Double
    fun player_pop_sample(player: Pointer): FFISample?
    fun sample_free(sample: FFISample)
    fun get_manifest_json(base_url: String): Pointer?
    fun free_string(string: Pointer)
    fun get_codec_config(base_url: String): FFICodecConfig?
    fun free_codec_config(config: FFICodecConfig)
    fun register_log_callback(callback: LogCallback)
}

interface LogCallback : com.sun.jna.Callback {
    fun invoke(msg: String)
}

class PlayerException(message: String) : Exception(message) {}

data class CodecConfig(
        val sps: ByteArray,
        val pps: ByteArray,
        val width: Int,
        val height: Int,
        val nalLengthSize: Int
)

object PlustCore {
    fun getCodecConfig(baseUrl: String): CodecConfig? {
        val ffi = PlustCoreLib.INSTANCE.get_codec_config(baseUrl) ?: return null
        val sps = ffi.sps_ptr?.getByteArray(0, ffi.sps_len.toInt()) ?: return null
        val pps = ffi.pps_ptr?.getByteArray(0, ffi.pps_len.toInt()) ?: return null
        val config =
                CodecConfig(
                        sps = sps,
                        pps = pps,
                        width = ffi.width,
                        height = ffi.height,
                        nalLengthSize = ffi.nal_length_size.toInt()
                )
        PlustCoreLib.INSTANCE.free_codec_config(ffi)
        return config
    }

    fun createPlayer(baseUrl: String): Pointer? {
        return PlustCoreLib.INSTANCE.player_new(baseUrl)
                ?: throw PlayerException("Failed to create player")
    }

    fun freePlayer(player: Pointer) {
        PlustCoreLib.INSTANCE.player_free(player)
    }

    fun play(player: Pointer) {
        val now = System.currentTimeMillis() / 1000.0
        PlustCoreLib.INSTANCE.player_play(player, now)
    }

    fun pause(player: Pointer) {
        val now = System.currentTimeMillis() / 1000.0
        PlustCoreLib.INSTANCE.player_pause(player, now)
    }

    fun seek(player: Pointer, media: Double) {
        val now = System.currentTimeMillis() / 1000.0
        PlustCoreLib.INSTANCE.player_seek(player, media, now)
    }

    fun isPlaying(player: Pointer): Boolean {
        return PlustCoreLib.INSTANCE.player_is_playing(player)
    }

    fun startLoading(player: Pointer) {
        PlustCoreLib.INSTANCE.player_start_loading(player)
    }

    fun popSample(player: Pointer): FFISample? {
        return PlustCoreLib.INSTANCE.player_pop_sample(player)
    }

    fun freeSample(sample: FFISample) {
        PlustCoreLib.INSTANCE.sample_free(sample)
    }

    fun freeString(string: Pointer) {
        PlustCoreLib.INSTANCE.free_string(string)
    }

    fun getMediaTime(player: Pointer, now: Double): Double {
        return PlustCoreLib.INSTANCE.player_get_media_time(player, now)
    }

    fun getManifestJson(baseUrl: String): String? {
        val ptr = PlustCoreLib.INSTANCE.get_manifest_json(baseUrl) ?: return null
        return try {
            ptr.getString(0)
        } finally {
            PlustCoreLib.INSTANCE.free_string(ptr)
        }
    }
}
