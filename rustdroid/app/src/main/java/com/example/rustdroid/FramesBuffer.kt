package com.example.rustdroid

import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

/**
 * Thread-safe buffer for GPU-stored frames, sorted by presentation time. When frames are evicted,
 * their texture slots are returned to the pool.
 */
class FramesBuffer(texturePool: TexturePool? = null) {
    private var texturePool: TexturePool? = texturePool
    private val samples = mutableListOf<GpuFrame>()
    private val lock = ReentrantLock()
    private var cursor: Int = 0

    /** Set the texture pool reference. Called after pool is created on GL thread. */
    fun setTexturePool(pool: TexturePool) {
        lock.withLock { this.texturePool = pool }
    }

    val count: Int
        get() = lock.withLock { samples.size }

    fun insert(sample: GpuFrame) {
        lock.withLock {
            val index = insertionIndex(sample.pts)
            samples.add(index, sample)
        }
    }

    fun popFirst(): GpuFrame? {
        return lock.withLock {
            if (samples.isEmpty()) null
            else {
                val frame = samples.removeAt(0)
                texturePool?.release(frame.poolSlot)
                frame
            }
        }
    }

    /**
     * Get the sample at or before the given time. Uses binary search for efficiency. Also evicts
     * frames that are too old (behind current time).
     * @param timeUs Time in microseconds
     */
    fun getSample(timeUs: Long): GpuFrame? {
        return lock.withLock {
            if (samples.isEmpty()) return@withLock null

            // Evict frames more than 0.5 seconds behind current time (from front, since sorted)
            val evictThresholdSecs = (timeUs - 500_000) / 1_000_000.0
            while (samples.isNotEmpty() && samples[0].pts < evictThresholdSecs) {
                val evicted = samples.removeAt(0)
                texturePool?.release(evicted.poolSlot)
            }

            if (samples.isEmpty()) return@withLock null

            val timeSecs = timeUs / 1_000_000.0

            var low = 0
            var high = samples.size - 1

            while (low < high) {
                val mid = (low + high + 1) / 2
                if (samples[mid].pts <= timeSecs) {
                    low = mid
                } else {
                    high = mid - 1
                }
            }

            if (samples[low].pts <= timeSecs) {
                samples[low]
            } else {
                null
            }
        }
    }

    fun clear() {
        lock.withLock {
            // Release all texture slots back to the pool
            samples.forEach { frame -> texturePool?.release(frame.poolSlot) }
            samples.clear()
            cursor = 0
        }
    }

    fun isEmpty(): Boolean {
        return lock.withLock { samples.isEmpty() }
    }

    fun getMaxPts(): Double? {
        return lock.withLock { samples.lastOrNull()?.pts }
    }

    /** Binary search to find insertion index maintaining sorted order by pts. */
    private fun insertionIndex(pts: Double): Int {
        var low = 0
        var high = samples.size
        while (low < high) {
            val mid = (low + high) / 2
            if (samples[mid].pts < pts) {
                low = mid + 1
            } else {
                high = mid
            }
        }
        return low
    }
}
