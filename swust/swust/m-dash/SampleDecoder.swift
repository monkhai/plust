import AVFoundation
import CoreVideo
import Metal
import VideoToolbox

struct DecodedSample {
	let pixelBuffer: CVPixelBuffer
	let pts: CMTime
}

struct Sample {
	let data: Data
	let pts: Double
	let dts: Double
	let duration: Double
	let isKey: Bool
	let isLast: Bool
}

final class SampleDecoder {
	private var formatDescription: CMFormatDescription?
	private var decompressionSession: VTDecompressionSession?
	private var textureCache: CVMetalTextureCache?
	private let device: MTLDevice
	private let samplesBuffer: SamplesBuffer
	private let timescale: CMTimeScale

	private var decodedPixelBuffer: CVPixelBuffer?

	init?(device: MTLDevice, sps: Data, pps: Data, buffer: SamplesBuffer, timescale: CMTimeScale) {
		self.device = device
		self.samplesBuffer = buffer
		self.timescale = timescale
		var cache: CVMetalTextureCache?
		CVMetalTextureCacheCreate(nil, nil, device, nil, &cache)
		guard let textureCache = cache else { return nil }
		self.textureCache = textureCache

		guard let formatDesc = createFormatDescription(sps: sps, pps: pps) else { return nil }
		self.formatDescription = formatDesc

		guard createDecompressionSession(formatDescription: formatDesc) else { return nil }
	}

	deinit {
		if let session = decompressionSession {
			VTDecompressionSessionInvalidate(session)
		}
	}

	// MARK: - Public

	func decoderSample(_ sample: Sample) {
		guard let formatDesc = formatDescription else { return }
		guard let block = self.createBlockBuffer(from: sample) else { return }
		guard
			let sampleBuffer = self.createSampleBuffer(
				from: block, sample: sample, formatDescription: formatDesc)
		else { return }
		self.decode(sampleBuffer: sampleBuffer)
	}

	// MARK: - Private

	private func decode(sampleBuffer: CMSampleBuffer) {
		guard let session = decompressionSession else { return }
		var infoFlags: VTDecodeInfoFlags = []
		VTDecompressionSessionDecodeFrame(
			session,
			sampleBuffer: sampleBuffer,
			flags: [._EnableAsynchronousDecompression],
			infoFlagsOut: &infoFlags,
			outputHandler: { _, _, imageBuffer, timestamp, _ in
				if let pixelBuffer = imageBuffer as CVPixelBuffer? {
					let pts = CMTime(seconds: CMTimeGetSeconds(timestamp), preferredTimescale: self.timescale)
					self.samplesBuffer.insert(DecodedSample(pixelBuffer: pixelBuffer, pts: pts))
				}
			}
		)
	}

	private func createSampleBuffer(
		from block: CMBlockBuffer,
		sample: Sample,
		formatDescription: CMFormatDescription
	) -> CMSampleBuffer? {
		var sampleBuffer: CMSampleBuffer?
		var sampleSize = sample.data.count

		let ptsCMTime = CMTime(seconds: sample.pts, preferredTimescale: self.timescale)
		let dtsCMTime = CMTime(seconds: sample.dts, preferredTimescale: self.timescale)
		let durationCMTime = CMTime(seconds: sample.duration, preferredTimescale: self.timescale)

		var timingInfo = CMSampleTimingInfo(
			duration: durationCMTime,
			presentationTimeStamp: ptsCMTime,
			decodeTimeStamp: dtsCMTime
		)

		CMSampleBufferCreateReady(
			allocator: kCFAllocatorDefault,
			dataBuffer: block,
			formatDescription: formatDescription,
			sampleCount: 1,
			sampleTimingEntryCount: 1,
			sampleTimingArray: &timingInfo,
			sampleSizeEntryCount: 1,
			sampleSizeArray: &sampleSize,
			sampleBufferOut: &sampleBuffer
		)

		return sampleBuffer
	}

	private func createBlockBuffer(from sample: Sample) -> CMBlockBuffer? {
		var blockBuffer: CMBlockBuffer?
		let dataPtr = UnsafeMutablePointer<UInt8>.allocate(capacity: sample.data.count)
		sample.data.copyBytes(to: dataPtr, count: sample.data.count)

		CMBlockBufferCreateWithMemoryBlock(
			allocator: nil,
			memoryBlock: dataPtr,
			blockLength: sample.data.count,
			blockAllocator: kCFAllocatorDefault,
			customBlockSource: nil,
			offsetToData: 0,
			dataLength: sample.data.count,
			flags: 0,
			blockBufferOut: &blockBuffer
		)

		return blockBuffer
	}

	private func createFormatDescription(sps: Data, pps: Data) -> CMFormatDescription? {
		return createH264FormatDescription(sps: sps, pps: pps)
	}

	private func createH264FormatDescription(sps: Data, pps: Data) -> CMFormatDescription? {

		var formatDescription: CMFormatDescription?
		sps.withUnsafeBytes { spsPtr in
			pps.withUnsafeBytes { ppsPtr in
				let parameterSetPointers: [UnsafePointer<UInt8>] = [
					spsPtr.baseAddress!.assumingMemoryBound(to: UInt8.self),
					ppsPtr.baseAddress!.assumingMemoryBound(to: UInt8.self),
				]
				let parameterSetSizes: [Int] = [sps.count, pps.count]

				CMVideoFormatDescriptionCreateFromH264ParameterSets(
					allocator: kCFAllocatorDefault,
					parameterSetCount: 2,
					parameterSetPointers: parameterSetPointers,
					parameterSetSizes: parameterSetSizes,
					nalUnitHeaderLength: 4,
					formatDescriptionOut: &formatDescription
				)
			}
		}

		return formatDescription
	}

	private func createDecompressionSession(formatDescription: CMFormatDescription) -> Bool {
		let destinationAttributes: [String: Any] = [
			kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
			kCVPixelBufferMetalCompatibilityKey as String: true,
		]

		var session: VTDecompressionSession?
		let status = VTDecompressionSessionCreate(
			allocator: kCFAllocatorDefault,
			formatDescription: formatDescription,
			decoderSpecification: nil,
			imageBufferAttributes: destinationAttributes as CFDictionary,
			outputCallback: nil,
			decompressionSessionOut: &session
		)

		if status != noErr { return false }

		self.decompressionSession = session
		return true
	}

	private func createFrame(from pixelBuffer: CVPixelBuffer, pts: CMTime) -> Frame? {
		guard let cache = textureCache else { return nil }

		let width = CVPixelBufferGetWidth(pixelBuffer)
		let height = CVPixelBufferGetHeight(pixelBuffer)

		// Y plane (luminance)
		var yTexture: CVMetalTexture?
		CVMetalTextureCacheCreateTextureFromImage(
			kCFAllocatorDefault,
			cache,
			pixelBuffer,
			nil,
			.r8Unorm,
			width,
			height,
			0,
			&yTexture
		)

		// UV plane (chrominance)
		var uvTexture: CVMetalTexture?
		CVMetalTextureCacheCreateTextureFromImage(
			kCFAllocatorDefault,
			cache,
			pixelBuffer,
			nil,
			.rg8Unorm,
			width / 2,
			height / 2,
			1,
			&uvTexture
		)

		guard let yCVTexture = yTexture,
			let uvCVTexture = uvTexture,
			let yMTLTexture = CVMetalTextureGetTexture(yCVTexture),
			let uvMTLTexture = CVMetalTextureGetTexture(uvCVTexture)
		else {
			return nil
		}

		return Frame(
			yTexture: yMTLTexture,
			uvTexture: uvMTLTexture,
			yCVTexture: yCVTexture,
			uvCVTexture: uvCVTexture,
			pts: pts
		)
	}
}
