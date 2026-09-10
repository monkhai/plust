import Foundation

enum FMP4 {
	static let boxChildOffsets: [String: Int] = [
		"moov": 8, "trak": 8,
		"mdia": 8, "minf": 8,
		"stbl": 8, "dinf": 8,
		"moof": 8, "traf": 8,
		"mvex": 8,
		"stsd": 16, "dref": 16,
		"avc1": 86, "hvc1": 86, "hev1": 86,
		"mp4a": 36,
	]

	static func getHeaderData(_ data: Data, at offset: Int) -> (size: UInt32, type: String)? {
		guard offset + 8 <= data.count else { return nil }
		let size =
			UInt32(data[offset]) << 24
			| UInt32(data[offset + 1]) << 16
			| UInt32(data[offset + 2]) << 8
			| UInt32(data[offset + 3])
		let type = String(data: data[offset + 4..<offset + 8], encoding: .ascii) ?? "????"
		return (size, type)
	}

	static func readUInt(_ data: Data, at offset: Int, for bytes: Int) -> UInt32 {
		var value: UInt32 = 0
		for i in 0..<bytes {
			let shift = (bytes - 1 - i) * 8
			value |= UInt32(data[offset + i]) << shift
		}
		return value
	}

	static func getAtom(_ data: Data, withPath path: [String]) -> Data? {
		guard !path.isEmpty else { return nil }

		var offset = 0
		var pathIndex = 0

		while offset + 8 <= data.count {
			guard let (size, type) = getHeaderData(data, at: offset) else { break }
			guard size > 0 else { break }

			if type == path[pathIndex] {
				if pathIndex == path.count - 1 {
					return Data(data[offset..<(offset + Int(size))])
				}
				let childOffset = boxChildOffsets[type] ?? 8
				offset += childOffset
				pathIndex += 1
			} else {
				offset += Int(size)
			}
		}

		return nil
	}
}
