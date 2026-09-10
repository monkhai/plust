import AVFoundation
import CoreVideo
import Foundation
import Metal

struct Frame {
	let yTexture: MTLTexture?
	let uvTexture: MTLTexture?
	let yCVTexture: CVMetalTexture?
	let uvCVTexture: CVMetalTexture?
	let pts: CMTime
}
