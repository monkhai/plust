/**
 * @typedef {Object} DecoderConfig
 * @property {Uint8Array} avcc_data - Raw avcC box data from init segment
 * @property {Uint8Array} sps - H.264 SPS (for codec string)
 * @property {number} width
 * @property {number} height
 */

/**
 * @typedef {Object} Sample
 * @property {Uint8Array} data - Compressed H.264 data
 * @property {number} pts - Presentation timestamp (seconds)
 * @property {number} dts - Decode timestamp (seconds)
 * @property {number} duration - Sample duration (seconds)
 * @property {boolean} is_key
 */

/**
 * @callback OnFrameCallback
 * @param {VideoFrame} videoFrame
 * @param {number} pts - Presentation timestamp in seconds
 */

/**
 * @callback OnErrorCallback
 * @param {Error} error
 */

/**
 * VideoDecoder - WebCodecs-based H.264 decoder
 * Like SampleDecoder.swift but using browser's WebCodecs API
 */
export class VideoDecoder {
    /** @type {globalThis.VideoDecoder | null} */
    #decoder = null

    /** @type {OnFrameCallback} */
    #onFrame

    /** @type {OnErrorCallback} */
    #onError

    /**
     * @param {DecoderConfig} config
     * @param {OnFrameCallback} onFrame
     * @param {OnErrorCallback} [onError]
     */
    constructor(config, onFrame, onError = console.error) {
        this.#onFrame = onFrame
        this.#onError = onError

        // Use the avcC data directly from Rust!
        // No need to reconstruct it - it's already the correct format
        const description = config.avcc_data

        // Configure WebCodecs VideoDecoder
        const decoderConfig = {
            codec: this.#getCodecString(config.sps),
            codedWidth: config.width,
            codedHeight: config.height,
            description: description,
            optimizeForLatency: true, // Like VideoToolbox async mode
        }

        this.#decoder = new globalThis.VideoDecoder({
            output: this.#handleFrame.bind(this),
            error: this.#handleError.bind(this),
        })

        this.#decoder.configure(decoderConfig)
    }

    /**
     * @param {Sample} sample
     */
    decode(sample) {
        if (!this.#decoder) {
            throw new Error('Decoder not initialized')
        }

        // WebCodecs expects timestamps in microseconds
        const timestamp = sample.pts * 1_000_000
        const duration = sample.duration * 1_000_000

        const chunk = new EncodedVideoChunk({
            type: sample.is_key ? 'key' : 'delta',
            timestamp: timestamp,
            duration: duration,
            data: sample.data,
        })

        this.#decoder.decode(chunk)
    }

    /** @returns {Promise<void>} */
    async flush() {
        if (this.#decoder) {
            await this.#decoder.flush()
        }
    }

    /** @returns {void} */
    close() {
        if (this.#decoder) {
            this.#decoder.close()
            this.#decoder = null
        }
    }

    /** @returns {'unconfigured' | 'configured' | 'closed'} */
    get state() {
        return this.#decoder?.state || 'closed'
    }

    // ============================================================================
    // PRIVATE METHODS
    // ============================================================================

    /** @param {VideoFrame} videoFrame */
    #handleFrame(videoFrame) {
        // Convert microseconds back to seconds for consistency
        const pts = videoFrame.timestamp / 1_000_000
        this.#onFrame(videoFrame, pts)
    }

    /** @param {Error} error */
    #handleError(error) {
        console.error('VideoDecoder error:', error)
        this.#onError(error)
    }

    // NOTE: We used to reconstruct the avcC box here from SPS/PPS,
    // but now Rust gives us the avcC data directly from the init segment.
    // Much cleaner!

    /**
     * @param {Uint8Array} sps
     * @returns {string} avc1.PPCCLL format
     */
    #getCodecString(sps) {
        // SPS bytes: [1] = profile, [2] = constraints, [3] = level
        const profile = sps[1].toString(16).padStart(2, '0')
        const constraints = sps[2].toString(16).padStart(2, '0')
        const level = sps[3].toString(16).padStart(2, '0')

        return `avc1.${profile}${constraints}${level}`
    }
}
