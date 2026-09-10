package com.example.rustdroid

/**
 * Represents a decoded video frame stored in GPU textures.
 * Unlike DecodedFrame which holds CPU ByteBuffers, this holds GL texture IDs.
 */
data class GpuFrame(
    val width: Int,
    val height: Int,
    val yTextureId: Int,
    val uvTextureId: Int,
    val pts: Double,
    val poolSlot: Int  // Index in TexturePool for recycling
)
