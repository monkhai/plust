import AVFoundation
import Foundation
import Metal

enum PlayerError: Error {
	case initFailed
	case noSample
	case noFrame
}

class Player {
	private var ptr: OpaquePointer
	private let samplesBuffer: SamplesBuffer
	private let decoder: SampleDecoder
	private let frameFactory: FrameFactory
	private var loadingTask: Task<(), Never>?
	private var currentFrame: Frame?
	private var nextFrame: Frame?
	private var manifest: Manifest

	private(set) var isPlaying: Bool = false

	init(url: String) throws {
		guard let ptr = player_new(url) else {
			Logger.log("no ptr")
			throw PlayerError.initFailed
		}
		self.ptr = ptr
		guard let device = MTLCreateSystemDefaultDevice() else {
			Logger.log("no device")
			throw PlayerError.initFailed
		}
		self.samplesBuffer = SamplesBuffer()
		let manifest = try Manifest.get(baseUrl: url)
		self.manifest = manifest
		self.frameFactory = try FrameFactory(device: device)
		guard let codecConfigPtr = get_codec_config(url) else {
			Logger.log("no codec config")
			throw PlayerError.initFailed
		}

		let codecConfig = codecConfigPtr.pointee
		let sps = Data(bytes: codecConfig.sps_ptr, count: codecConfig.sps_len)
		let pps = Data(bytes: codecConfig.pps_ptr, count: codecConfig.pps_len)

		free_codec_config(codecConfigPtr)

		guard
			let decoder = SampleDecoder(
				device: device,
				sps: sps,
				pps: pps,
				buffer: self.samplesBuffer,
				timescale: CMTimeScale(manifest.segmentTemplate.timescale),
			)
		else {
			Logger.log("no decoder")
			throw PlayerError.initFailed
		}
		self.decoder = decoder
		self.startSampleLoading()
		self.startFrameDecoding()
	}

	deinit {
		loadingTask?.cancel()
		player_free(self.ptr)
	}

	func getCurrentFrame() -> Frame? {
		// if we are not playing
		//  we return the current frame (or get it)
		//  or throw if we don't have any frames yet
		if !self.isPlaying {
			if let frame = self.currentFrame {
				return frame
			} else if let frame = try? self.getNextFrame() {
				self.currentFrame = frame
				return frame
			}
			return nil
		}

		// if we don't have enough samples yet
		// we keep the player at 0 and try to get and return a frame
		if self.samplesBuffer.count < 30 {
			player_seek(self.ptr, 0.0, now())
			if let frame = currentFrame {
				self.nextFrame = try? self.getNextFrame()
				return frame
			} else if let frame = try? self.getNextFrame() {
				self.currentFrame = frame
				self.nextFrame = try? self.getNextFrame()
				return frame
			} else {
				return nil
			}
		}

		guard let current = self.currentFrame else {
			self.currentFrame = try? self.getNextFrame()
			return nil
		}

		// here we defo have a current frame
		guard let next = self.nextFrame else {
			self.nextFrame = try? self.getNextFrame()
			return current
		}

		let media_time = self.getMediaTime()

		if media_time >= next.pts.seconds {
			self.currentFrame = next
			self.nextFrame = try? self.getNextFrame()
		}

		guard let frame = self.currentFrame else { return nil }
		return frame
	}

	func play() {
		player_play(self.ptr, now())
		self.isPlaying = true
	}

	func pause() {
		player_pause(self.ptr, now())
		self.isPlaying = false
	}

	func seek(to time: Double) {
		player_seek(self.ptr, time, now())
	}

	func startSampleLoading() {
		player_start_loading(self.ptr)
	}

	func startFrameDecoding() {
		self.loadingTask = Task.detached { [weak self] in
			while !Task.isCancelled {
				guard let self = self else { return }
				guard let sample = try? await self.popSample() else {
					try? await Task.sleep(for: .milliseconds(10))
					continue
				}
				await self.decoder.decoderSample(sample)
			}
		}
	}

	func setPlaybackRate(toRate rate: Double) {
		player_set_playback_rate(self.ptr, rate, now())
	}

	private func popSample() throws -> Sample {
		guard let samplePtr = player_pop_sample(self.ptr) else {
			throw PlayerError.noSample
		}

		defer { sample_free(samplePtr) }

		let s = samplePtr.pointee
		let data = Data(bytes: s.data_ptr, count: s.data_len)
		return Sample(
			data: data,
			pts: s.pts,
			dts: s.dts,
			duration: s.duration,
			isKey: s.is_key,
			isLast: s.is_last
		)
	}

	private func getNextFrame() throws -> Frame {
		if self.samplesBuffer.isEmpty() { throw PlayerError.noSample }
		guard let sample = self.samplesBuffer.getNextSample() else { throw PlayerError.noSample }
		guard let frame = frameFactory.createFrame(from: sample) else { throw PlayerError.noFrame }
		return frame
	}

	private func getMediaTime() -> Double {
		return player_get_media_time(self.ptr, now())
	}
}

private func now() -> Double {
	return Date().timeIntervalSince1970
}
