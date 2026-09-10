package com.example.rustdroid

/**
 * Raw encoded sample from the Rust demuxer. This is the Kotlin equivalent of the FFISample struct.
 */
data class Sample(
        val data: ByteArray,
        val pts: Double,
        val dts: Double,
        val duration: Double,
        val isKey: Boolean
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Sample

        if (!data.contentEquals(other.data)) return false
        if (pts != other.pts) return false
        if (dts != other.dts) return false
        if (duration != other.duration) return false
        if (isKey != other.isKey) return false

        return true
    }

    override fun hashCode(): Int {
        var result = data.contentHashCode()
        result = 31 * result + pts.hashCode()
        result = 31 * result + dts.hashCode()
        result = 31 * result + duration.hashCode()
        result = 31 * result + isKey.hashCode()
        return result
    }
}
