//@ts-check

import { PlustCore } from './PlustCore.js'
import { Player } from './Player.js'

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById('canvas')

async function main() {
    console.log('🚀 Initializing PlustCore...')
    await PlustCore.init()

    console.log('🎮 Creating player...')
    const player = new Player('http://localhost:8080')

    try {
        await player.init(canvas)
        console.log('✅ Player initialized!')
        console.log('Canvas size:', canvas.width, 'x', canvas.height)
    } catch (error) {
        console.error('❌ Player init failed:', error)
        throw error
    }

    // Play immediately - decoding will start and frames will buffer in real-time
    console.log('\n▶️  Starting playback...')
    player.play()

    console.log('✅ Video should be playing!')

    // Expose to console
    window.player = player
}

main().catch(console.error)
