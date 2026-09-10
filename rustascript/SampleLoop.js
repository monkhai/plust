//@ts-check

/**
 * SampleLoop - Background loop that continuously decodes samples
 *
 * Like Swift's startFrameDecoding() task (Player.swift lines 140-151)
 */
export class SampleLoop {
    /** @type {boolean} */
    #running = false

    /** @type {import('./Player.js').Player} */
    #player

    /** @type {import('./VideoDecoder.js').VideoDecoder} */
    #decoder

    /**
     * @param {import('./Player.js').Player} player
     * @param {import('./VideoDecoder.js').VideoDecoder} decoder
     */
    constructor(player, decoder) {
        this.#player = player
        this.#decoder = decoder
    }

    #samplesDecoded = 0

    /**
     * Start the decoding loop
     */
    start() {
        if (this.#running) {
            console.log('[SampleLoop] Already running')
            return
        }
        console.log('[SampleLoop] Starting...')
        this.#running = true
        this.#loop()
    }

    /**
     * Stop the decoding loop
     */
    stop() {
        this.#running = false
    }

    /**
     * Main decoding loop - runs continuously in background
     * @private
     */
    async #loop() {
        while (this.#running) {
            // Backpressure: pause decoding if buffer is too full
            // This prevents decoding too far ahead before playback starts
            const bufferSize = this.#player.bufferSize
            if (bufferSize >= 100) {
                await this.#sleep(50)
                continue
            }

            // Try to get next sample
            const sample = this.#player.popSample()

            if (sample) {
                // Decode it (non-blocking - callback will handle result)
                try {
                    this.#decoder.decode(sample)
                    this.#samplesDecoded++

                    // Log every 10 samples
                    if (this.#samplesDecoded % 10 === 0) {
                        console.log('[SampleLoop] Decoded', this.#samplesDecoded, 'samples, buffer:', bufferSize)
                    }

                    // Small delay to prevent flooding the decoder queue
                    // This gives the render loop time to consume frames
                    await this.#sleep(1)
                } catch (error) {
                    console.error('[SampleLoop] Decode error:', error)
                }
            } else {
                // No samples available, wait a bit
                await this.#sleep(10)
            }
        }
    }

    /**
     * Sleep helper
     * @param {number} ms
     * @returns {Promise<void>}
     * @private
     */
    #sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms))
    }
}
