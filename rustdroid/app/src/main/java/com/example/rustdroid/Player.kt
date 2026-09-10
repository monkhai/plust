package com.example.rustdroid

import android.opengl.GLSurfaceView
import com.sun.jna.Pointer
import java.io.Closeable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class Player(url: String) : Closeable {
    var isPlaying: Boolean = false

    private var ptr: Pointer? = null
    private var decodingJob: Job? = null
    private val framesBuffer: FramesBuffer = FramesBuffer()
    private val decoder: SampleDecoder = SampleDecoder(url)

    // GL thread coordination
    private var glSurfaceView: GLSurfaceView? = null
    private var videoRenderer: VideoRenderer? = null
    private var texturePoolInitialized = false

    init {
        val tempPtr = PlustCore.createPlayer(url)
        try {
            Manifest.get(url) ?: throw PlayerException("Failed to get manifest")
            ptr = tempPtr
            startLoading()
        } catch (e: Exception) {
            tempPtr?.let { PlustCore.freePlayer(it) }
            throw e
        }
    }

    /**
     * Set the GLSurfaceView and VideoRenderer for GL thread coordination. Must be called before
     * startFrameDecoding.
     */
    fun setGLContext(surfaceView: GLSurfaceView, renderer: VideoRenderer) {
        this.glSurfaceView = surfaceView
        this.videoRenderer = renderer
    }

    override fun close() {
        ptr?.let { PlustCore.freePlayer(it) }
        ptr = null
        stopFrameDecoding()
    }

    fun getCurrentFrame(): GpuFrame? {
        // Wait for enough frames before starting
        if (this.framesBuffer.count < 30) {
            PlustCore.seek(this.ptr!!, 0.0)
            return framesBuffer.getSample(0)
        }

        // Get frame at current playback time
        val mediaTime = this.getMediaTime()
        val timeUs = (mediaTime * 1_000_000).toLong()
        return framesBuffer.getSample(timeUs)
    }

    fun play() {
        ptr?.let { player ->
            PlustCore.play(player)
            isPlaying = PlustCore.isPlaying(player)
        }
    }

    fun pause() {
        ptr?.let { player ->
            PlustCore.pause(player)
            isPlaying = PlustCore.isPlaying(player)
        }
    }

    fun getMediaTime(): Double {
        return ptr?.let { player -> PlustCore.getMediaTime(player, now()) } ?: 0.0
    }

    private fun startLoading() {
        ptr?.let { PlustCore.startLoading(it) }
    }

    fun startFrameDecoding() {
        if (ptr == null) return
        val surfaceView = glSurfaceView ?: return
        val renderer = videoRenderer ?: return

        decodingJob =
                CoroutineScope(Dispatchers.IO).launch {
                    while (isActive) {
                        // Throttle: don't decode more than 2 seconds ahead of playback
                        val maxPts = framesBuffer.getMaxPts()
                        val currentTime = getMediaTime()
                        if (isPlaying && maxPts != null && maxPts - currentTime > 2.0) {
                            delay(5)
                            continue
                        }

                        val sample = popSample()
                        if (sample == null) {
                            delay(10)
                            continue
                        }
                        val decodedFrame = decoder.decode(sample)
                        if (decodedFrame == null) {
                            delay(10)
                            continue
                        }

                        // Upload frame on GL thread
                        surfaceView.queueEvent {
                            val gpuFrame = renderer.uploadFrame(decodedFrame)
                            if (gpuFrame != null) {
                                // Set texture pool on buffer if not done yet
                                if (!texturePoolInitialized) {
                                    renderer.getTexturePool()?.let { pool ->
                                        framesBuffer.setTexturePool(pool)
                                        texturePoolInitialized = true
                                    }
                                }
                                framesBuffer.insert(gpuFrame)
                            }
                            // DecodedFrame's ByteBuffers are now eligible for GC
                        }
                    }
                }
    }

    private fun stopFrameDecoding() {
        decodingJob?.cancel()
        decodingJob = null
    }

    fun popSample(): Sample? {
        val ffi = ptr?.let { PlustCore.popSample(it) } ?: return null
        val data = ffi.data_ptr?.getByteArray(0, ffi.data_len.toInt()) ?: return null
        val sample =
                Sample(
                        data = data,
                        pts = ffi.pts,
                        dts = ffi.dts,
                        duration = ffi.duration,
                        isKey = ffi.is_key != 0.toByte()
                )
        PlustCore.freeSample(ffi)
        return sample
    }

    private fun now(): Double {
        return System.currentTimeMillis() / 1000.0
    }
}
