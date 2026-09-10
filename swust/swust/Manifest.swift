import Foundation

struct Mpd: Codable {
	let mediaPresentationDuration: String

	enum CodingKeys: String, CodingKey {
		case mediaPresentationDuration = "media_presentation_duration"
	}
}

struct Representation: Codable {
	let id: String
	let bandwidth: Int
	let codecs: String
	let height: String
	let width: String
}

struct SegmentTemplate: Codable {
	let initialization: String
	let startNumber: Int
	let media: String
	let timescale: Int
	let duration: String

	enum CodingKeys: String, CodingKey {
		case initialization
		case startNumber = "start_number"
		case media
		case timescale
		case duration
	}
}

struct Manifest: Codable {
	let mpd: Mpd
	let baseUrl: String
	let representations: [Representation]
	let segmentTemplate: SegmentTemplate
	let segmentCount: Int

	enum CodingKeys: String, CodingKey {
		case mpd
		case baseUrl = "base_url"
		case representations
		case segmentTemplate = "segment_template"
		case segmentCount = "segment_count"
	}

	static func get(baseUrl: String) throws -> Manifest {
		guard let jsonPtr = get_manifest_json(baseUrl) else {
			throw NSError(
				domain: "Manifest",
				code: 1,
				userInfo: [NSLocalizedDescriptionKey: "Failed to get manifest"]
			)
		}
		defer { free_string(UnsafeMutablePointer(mutating: jsonPtr)) }
		let json = String(cString: jsonPtr)
		return try JSONDecoder().decode(Manifest.self, from: Data(json.utf8))
	}
}
