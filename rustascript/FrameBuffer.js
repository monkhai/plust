//@ts-check

export class FrameBuffer {
    // TODO: Private field to store frames
    // Each frame is: { videoFrame: VideoFrame, pts: number }
    // Hint: Use an array, keep it sorted by PTS
    /** @type {Array<{ videoFrame: VideoFrame, pts: number }>} */
    #frames;
    /** @type {number} */
    #maxSize;

    /**
     * @param {number} maxSize - Maximum number of frames to store
     */
    constructor(maxSize = 100) {
        // TODO: Initialize your private fields
        this.#frames = [];
        this.#maxSize = maxSize;
    }

    /**
     * @param {VideoFrame} videoFrame - The decoded frame
     * @param {number} pts - Presentation timestamp in seconds
     */
    insert(videoFrame, pts) {
        const index = this.#insertionIndex(pts);
        this.#frames.splice(index, 0, {videoFrame, pts});
        if (this.#frames.length > this.#maxSize) {
            const frame = this.#frames.shift();
            frame?.videoFrame.close();
        }
    }

    /**
     * @param {number} time - Current playback time in seconds
     * @returns {VideoFrame | null} - The frame to display, or null if none available
     */
    getFrameAt(time) {
        if (this.#frames.length == 0) {
            return null;
        }

        let low = 0
        let high = this.#frames.length - 1

        while (low < high) {
            let mid = Math.floor((low + high + 1) / 2);
            if (this.#frames[mid].pts <= time) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        if (this.#frames[low].pts <= time) {
            return this.#frames[low].videoFrame;
        }
        return null;
    }

    clear() {
        this.#frames.forEach((frame) => {
            frame.videoFrame.close();
        })
        this.#frames = [];
    }

    /** @returns {number} */
    get length() {
        return this.#frames.length;
    }

    /** @returns {boolean} */
    isEmpty() {
        return this.#frames.length === 0;
    }

    /**
     * Remove frames older than the given time (they've been consumed)
     * @param {number} time - Current playback time
     */
    removeOldFrames(time) {
        // Keep frames that are within 1 second behind current time
        const threshold = time - 1.0

        while (this.#frames.length > 0 && this.#frames[0].pts < threshold) {
            const frame = this.#frames.shift()
            frame?.videoFrame.close()
        }
    }

    /**
     * Get PTS range in buffer (for debugging)
     * @returns {{ min: number, max: number } | null}
     */
    getPTSRange() {
        if (this.#frames.length === 0) return null;
        return {
            min: this.#frames[0].pts,
            max: this.#frames[this.#frames.length - 1].pts
        };
    }

    #insertionIndex(pts) {
        let low = 0;
        let high = this.#frames.length;
        while (low < high) {
            let mid = Math.floor((low + high) / 2);
            if (this.#frames[mid].pts < pts) {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        return low;
    }
}
