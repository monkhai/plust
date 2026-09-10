//@ts-check

/**
 * RenderLoop - Continuously render frames to canvas synced with playback time
 *
 * Like Swift's MetalRenderer draw() method (MetalRenderer.swift lines 27-50)
 * combined with getCurrentFrame() logic (Player.swift lines 70-120)
 */
export class RenderLoop {
    /** @type {boolean} */
    #running = false

    /** @type {number | null} */
    #animationId = null

    /** @type {import('./Player.js').Player} */
    #player

    /** @type {import('./FrameBuffer.js').FrameBuffer} */
    #frameBuffer

    /** @type {HTMLCanvasElement} */
    #canvas

    /** @type {CanvasRenderingContext2D} */
    #ctx

    /**
     * @param {import('./Player.js').Player} player
     * @param {import('./FrameBuffer.js').FrameBuffer} frameBuffer
     * @param {HTMLCanvasElement} canvas
     */
    constructor(player, frameBuffer, canvas) {
        this.#player = player
        this.#frameBuffer = frameBuffer
        this.#canvas = canvas
        this.#ctx = canvas.getContext('2d')
    }

    /**
     * Start the render loop
     */
    start() {
        if (this.#running) {
            console.log('[RenderLoop] Already running')
            return
        }
        console.log('[RenderLoop] Starting...')
        this.#running = true
        this.#render()
    }

    /**
     * Stop the render loop
     */
    stop() {
        this.#running = false
        if (this.#animationId !== null) {
            cancelAnimationFrame(this.#animationId)
            this.#animationId = null
        }
    }

    #frameCount = 0

    /**
     * Main render loop - runs at display refresh rate (~60fps)
     * @private
     */
    #render = () => {
        if (!this.#running) return

        this.#frameCount++

        // Get current playback time
        const mediaTime = this.#player.getMediaTime()

        // Remove old frames that are no longer needed
        // This frees up space in the buffer for new frames
        this.#frameBuffer.removeOldFrames(mediaTime)

        // Get frame that should be displayed at this time
        const frame = this.#frameBuffer.getFrameAt(mediaTime)

        // Log every 60 frames (~1 second)
        if (this.#frameCount % 60 === 0) {
            const ptsRange = this.#frameBuffer.getPTSRange()
            const rangeStr = ptsRange ? `${ptsRange.min.toFixed(3)}-${ptsRange.max.toFixed(3)}` : 'empty'
            console.log('[RenderLoop] Frame', this.#frameCount, 'time:', mediaTime.toFixed(3), 'frame:', frame ? 'found' : 'null', 'bufferSize:', this.#frameBuffer.length, 'bufferPTS:', rangeStr)
        }

        if (frame) {
            // Draw frame to canvas
            this.#ctx.drawImage(frame, 0, 0, this.#canvas.width, this.#canvas.height)
        }

        // Schedule next frame
        this.#animationId = requestAnimationFrame(this.#render)
    }
}
