import { PlustCore } from './PlustCore.js'
import { VideoDecoder } from './VideoDecoder.js'
import { FrameBuffer } from './FrameBuffer.js'
import { SampleLoop } from './SampleLoop.js'
import { RenderLoop } from './RenderLoop.js'

/**
 * Player - High-level class that manages a player instance
 * Orchestrates WASM player, decoder, frame buffer, and rendering
 */
export class Player {
    /** @type {string} */
    baseUrl

    /** @type {number | null} */
    #playerId = null

    /** @type {VideoDecoder | null} */
    #decoder = null

    /** @type {FrameBuffer | null} */
    #frameBuffer = null

    /** @type {SampleLoop | null} */
    #sampleLoop = null

    /** @type {RenderLoop | null} */
    #renderLoop = null

    /** @type {HTMLCanvasElement | null} */
    #canvas = null

    /** @param {string} baseUrl */
    constructor(baseUrl) {
        this.baseUrl = baseUrl
    }

    /**
     * Initialize the player
     * @param {HTMLCanvasElement} canvas - Canvas element to render to
     * @returns {Promise<Player>}
     */
    async init(canvas) {
        console.log('[Player] Creating WASM player...')
        this.#playerId = await PlustCore.playerNew(this.baseUrl)
        console.log('[Player] Player ID:', this.#playerId)

        // Store canvas
        this.#canvas = canvas
        console.log('[Player] Canvas element:', canvas)

        // Get codec config
        console.log('[Player] Getting codec config...')
        const config = await PlustCore.getCodecConfig(this.baseUrl)
        console.log('[Player] Config:', config)

        // Set canvas size to match video
        canvas.width = config.width
        canvas.height = config.height
        console.log('[Player] Canvas sized to:', canvas.width, 'x', canvas.height)

        // Create frame buffer (keep ~5 seconds at 30fps = 150 frames)
        this.#frameBuffer = new FrameBuffer(150)
        console.log('[Player] Frame buffer created')

        // Create decoder with callback
        this.#decoder = new VideoDecoder(
            config,
            (videoFrame, pts) => {
                // When frame is decoded, insert into buffer
                this.#frameBuffer.insert(videoFrame, pts)
            },
            (error) => {
                console.error('Decoder error:', error)
            }
        )
        console.log('[Player] Decoder created, state:', this.#decoder.state)

        // Create sample loop (continuously decodes samples)
        this.#sampleLoop = new SampleLoop(this, this.#decoder)
        console.log('[Player] Sample loop created')

        // Create render loop (continuously renders frames)
        this.#renderLoop = new RenderLoop(this, this.#frameBuffer, canvas)
        console.log('[Player] Render loop created')

        // Start loading segments
        PlustCore.playerStartLoading(this.#playerId)
        console.log('[Player] Loading started')

        // DON'T start decoding yet - wait until play() is called
        // This prevents buffer from filling with later frames before playback starts
        console.log('[Player] Ready to play (decoding will start on play())')

        return this
    }

    /**
     * Free the player and cleanup resources
     */
    free() {
        // Stop loops
        if (this.#sampleLoop) {
            this.#sampleLoop.stop()
            this.#sampleLoop = null
        }

        if (this.#renderLoop) {
            this.#renderLoop.stop()
            this.#renderLoop = null
        }

        // Close decoder
        if (this.#decoder) {
            this.#decoder.close()
            this.#decoder = null
        }

        // Clear frame buffer
        if (this.#frameBuffer) {
            this.#frameBuffer.clear()
            this.#frameBuffer = null
        }

        // Free WASM player
        if (this.#playerId !== null) {
            PlustCore.playerFree(this.#playerId)
            this.#playerId = null
        }
    }

    // ============================================================================
    // PLAYER CONTROL
    // ============================================================================

    play() {
        this.#assertInitialized()
        console.log('[Player] Playing at time:', now())
        PlustCore.playerPlay(this.#playerId, now())

        // Start decoding (if not already started)
        if (this.#sampleLoop) {
            console.log('[Player] Starting sample loop')
            this.#sampleLoop.start()
        }

        // Start rendering
        if (this.#renderLoop) {
            console.log('[Player] Starting render loop')
            this.#renderLoop.start()
        } else {
            console.error('[Player] No render loop available!')
        }
    }

    pause() {
        this.#assertInitialized()
        PlustCore.playerPause(this.#playerId, now())

        // Stop rendering (but keep decoding)
        if (this.#renderLoop) {
            this.#renderLoop.stop()
        }
    }

    seek(mediaTime) {
        this.#assertInitialized()
        PlustCore.playerSeek(this.#playerId, mediaTime, now())
    }

    setPlaybackRate(rate) {
        this.#assertInitialized()
        PlustCore.playerSetPlaybackRate(this.#playerId, rate, now())
    }

    // ============================================================================
    // PLAYER STATE
    // ============================================================================

    isPlaying() {
        if (this.#playerId === null) return false
        return PlustCore.playerIsPlaying(this.#playerId)
    }

    getMediaTime() {
        if (this.#playerId === null) return 0.0
        return PlustCore.playerGetMediaTime(this.#playerId, now())
    }

    get bufferSize() {
        return this.#frameBuffer?.length || 0
    }

    // ============================================================================
    // SAMPLES (Low-level access - usually not needed)
    // ============================================================================

    popSample() {
        this.#assertInitialized()
        return PlustCore.playerPopSample(this.#playerId)
    }

    // ============================================================================
    // HELPERS
    // ============================================================================

    #assertInitialized() {
        if (this.#playerId === null) {
            throw new Error('Player not initialized. Call init() first.')
        }
    }
}

function now() {
    return PlustCore.now()
}
