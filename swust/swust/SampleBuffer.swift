import CoreMedia

class SamplesBuffer {
	private var samples: [DecodedSample] = []
	private let lock = NSLock()
	private var cursor: Int = 0

	var count: Int {
		lock.lock()
		defer { lock.unlock() }
		return samples.count
	}

	func insert(_ sample: DecodedSample) {
		lock.lock()
		defer { lock.unlock() }
		let index = insertionIndex(for: sample.pts)
		samples.insert(sample, at: index)
	}

	func popFirst() -> DecodedSample? {
		lock.lock()
		defer { lock.unlock() }
		guard !samples.isEmpty else { return nil }
		return samples.removeFirst()
	}

	func getNextSample() -> DecodedSample? {
		lock.lock()
		defer { lock.unlock() }
		// make sure to reset the cursor to 0 if we are at the end of the buffer
		if self.cursor >= samples.count {
			self.cursor = 0
		}
		guard !self.samples.isEmpty else { return nil }
		let sample = self.samples[self.cursor]
		self.cursor += 1
		return sample
	}

	func getSample(at time: CMTime) -> DecodedSample? {
		lock.lock()
		defer { lock.unlock() }
		guard !samples.isEmpty else { return nil }

		var low = 0
		var high = samples.count - 1

		while low < high {
			let mid = (low + high + 1) / 2
			if CMTimeCompare(samples[mid].pts, time) <= 0 {
				low = mid
			} else {
				high = mid - 1
			}
		}

		if CMTimeCompare(samples[low].pts, time) <= 0 {
			return samples[low]
		}

		return nil
	}

	func clear() {
		lock.lock()
		defer { lock.unlock() }
		samples.removeAll()
	}

	func isEmpty() -> Bool {
		lock.lock()
		defer { lock.unlock() }
		return samples.isEmpty
	}

	private func insertionIndex(for pts: CMTime) -> Int {
		var low = 0
		var high = samples.count
		while low < high {
			let mid = (low + high) / 2
			if CMTimeCompare(samples[mid].pts, pts) < 0 {
				low = mid + 1
			} else {
				high = mid
			}
		}
		return low
	}
}
