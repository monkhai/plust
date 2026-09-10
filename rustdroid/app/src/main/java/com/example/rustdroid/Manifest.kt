package com.example.rustdroid

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class Mpd(@SerialName("media_presentation_duration") val mediaPresentationDuration: String)

@Serializable
data class Representation(
        val id: String,
        val bandwidth: Int,
        val codecs: String,
        val height: String,
        val width: String
)

@Serializable
data class SegmentTemplate(
        val initialization: String,
        @SerialName("start_number") val startNumber: Int,
        val media: String,
        val timescale: Int,
        val duration: String
)

@Serializable
data class Manifest(
        val mpd: Mpd,
        @SerialName("base_url") val baseUrl: String,
        val representations: List<Representation>,
        @SerialName("segment_template") val segmentTemplate: SegmentTemplate,
        @SerialName("segment_count") val segmentCount: Int
) {
    companion object {
        private val json = Json { ignoreUnknownKeys = true }

        fun get(baseUrl: String): Manifest? {
            val jsonString = PlustCore.getManifestJson(baseUrl) ?: return null
            return try {
                json.decodeFromString<Manifest>(jsonString)
            } catch (e: Exception) {
                null
            }
        }
    }
}
