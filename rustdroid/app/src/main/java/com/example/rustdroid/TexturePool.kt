package com.example.rustdroid

import android.opengl.GLES20
import java.nio.ByteBuffer
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

/**
 * Pool of pre-allocated OpenGL textures for efficient frame storage.
 * Textures are allocated once and reused to avoid per-frame allocation overhead.
 * All GL operations must be called on the GL thread.
 */
class TexturePool(
    private val capacity: Int,
    private val width: Int,
    private val height: Int
) {
    private val yTextureIds = IntArray(capacity)
    private val uvTextureIds = IntArray(capacity)
    private val available = ArrayDeque<Int>()
    private val lock = ReentrantLock()
    private var initialized = false

    /**
     * Initialize all textures. Must be called on GL thread after EGL context is ready.
     */
    fun initialize() {
        if (initialized) return

        // Generate Y plane textures
        GLES20.glGenTextures(capacity, yTextureIds, 0)
        // Generate UV plane textures
        GLES20.glGenTextures(capacity, uvTextureIds, 0)

        for (i in 0 until capacity) {
            // Configure Y texture (luminance, full size)
            configureTexture(yTextureIds[i])
            GLES20.glTexImage2D(
                GLES20.GL_TEXTURE_2D,
                0,
                GLES20.GL_LUMINANCE,
                width,
                height,
                0,
                GLES20.GL_LUMINANCE,
                GLES20.GL_UNSIGNED_BYTE,
                null
            )

            // Configure UV texture (luminance-alpha, half size)
            configureTexture(uvTextureIds[i])
            GLES20.glTexImage2D(
                GLES20.GL_TEXTURE_2D,
                0,
                GLES20.GL_LUMINANCE_ALPHA,
                width / 2,
                height / 2,
                0,
                GLES20.GL_LUMINANCE_ALPHA,
                GLES20.GL_UNSIGNED_BYTE,
                null
            )

            // Mark slot as available
            available.addLast(i)
        }

        initialized = true
    }

    /**
     * Acquire an available texture slot. Returns null if pool is exhausted.
     */
    fun acquire(): Int? {
        return lock.withLock {
            if (available.isEmpty()) null else available.removeFirst()
        }
    }

    /**
     * Release a texture slot back to the pool for reuse.
     */
    fun release(slot: Int) {
        lock.withLock {
            if (slot in 0 until capacity && slot !in available) {
                available.addLast(slot)
            }
        }
    }

    /**
     * Upload frame data to the textures at the given slot.
     * Must be called on GL thread.
     */
    fun uploadFrame(slot: Int, yData: ByteBuffer, uvData: ByteBuffer) {
        if (slot !in 0 until capacity) return

        // Upload Y plane
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, yTextureIds[slot])
        yData.position(0)
        GLES20.glTexSubImage2D(
            GLES20.GL_TEXTURE_2D,
            0,
            0, 0,
            width,
            height,
            GLES20.GL_LUMINANCE,
            GLES20.GL_UNSIGNED_BYTE,
            yData
        )

        // Upload UV plane
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, uvTextureIds[slot])
        uvData.position(0)
        GLES20.glTexSubImage2D(
            GLES20.GL_TEXTURE_2D,
            0,
            0, 0,
            width / 2,
            height / 2,
            GLES20.GL_LUMINANCE_ALPHA,
            GLES20.GL_UNSIGNED_BYTE,
            uvData
        )
    }

    fun getYTextureId(slot: Int): Int = yTextureIds[slot]
    fun getUVTextureId(slot: Int): Int = uvTextureIds[slot]

    fun availableCount(): Int = lock.withLock { available.size }

    /**
     * Destroy all textures. Must be called on GL thread.
     */
    fun destroy() {
        if (!initialized) return
        GLES20.glDeleteTextures(capacity, yTextureIds, 0)
        GLES20.glDeleteTextures(capacity, uvTextureIds, 0)
        lock.withLock { available.clear() }
        initialized = false
    }

    private fun configureTexture(textureId: Int) {
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_MIN_FILTER, GLES20.GL_LINEAR)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_MAG_FILTER, GLES20.GL_LINEAR)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_WRAP_S, GLES20.GL_CLAMP_TO_EDGE)
        GLES20.glTexParameteri(GLES20.GL_TEXTURE_2D, GLES20.GL_TEXTURE_WRAP_T, GLES20.GL_CLAMP_TO_EDGE)
    }
}
