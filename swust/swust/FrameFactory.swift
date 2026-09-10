import AVFoundation
import Metal

enum FrameFactoryError: Error {
	case noDevice
	case noCache
}

class FrameFactory {
	private let device: MTLDevice
	private let cache: CVMetalTextureCache

	init(device: MTLDevice) throws {
		self.device = device
		var cache: CVMetalTextureCache?
		let rc = CVMetalTextureCacheCreate(nil, nil, device, nil, &cache)
		guard rc == kCVReturnSuccess, let cache else { throw FrameFactoryError.noCache }
		self.cache = cache
	}

	func createFrame(from decodedSample: DecodedSample) -> Frame? {
		let pixelBuffer = decodedSample.pixelBuffer
		let pts = decodedSample.pts

		var yTexture: MTLTexture?
		var uvTexture: MTLTexture?
		var yCvTexture: CVMetalTexture?
		var uvCvTexture: CVMetalTexture?

		let planeCount = CVPixelBufferGetPlaneCount(pixelBuffer)

		for i in 0..<planeCount {
			let w = CVPixelBufferGetWidthOfPlane(pixelBuffer, i)
			let h = CVPixelBufferGetHeightOfPlane(pixelBuffer, i)
			let format: MTLPixelFormat = (i == 0) ? .r8Unorm : .rg8Unorm
			var cvTexture: CVMetalTexture?
			let status = CVMetalTextureCacheCreateTextureFromImage(
				nil, cache, pixelBuffer, nil, format, w, h, i, &cvTexture
			)

			guard status == kCVReturnSuccess, let cvTexture else { return nil }
			guard let texture = CVMetalTextureGetTexture(cvTexture) else { return nil }

			if i == 0 {
				yTexture = texture
				yCvTexture = cvTexture
			}
			if i == 1 {
				uvTexture = texture
				uvCvTexture = cvTexture
			}
		}

		return Frame(
			yTexture: yTexture,
			uvTexture: uvTexture,
			yCVTexture: yCvTexture,
			uvCVTexture: uvCvTexture,
			pts: pts
		)
	}

}
