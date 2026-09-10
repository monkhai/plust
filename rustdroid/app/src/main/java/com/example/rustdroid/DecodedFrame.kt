package com.example.rustdroid

import java.nio.ByteBuffer

data class DecodedFrame(
        val width: Int,
        val height: Int,
        val yPlane: ByteBuffer,
        val uvPlane: ByteBuffer,
        val pts: Double,
)
