import Foundation

private func rustLogHandler(msg: UnsafePointer<CChar>?) {
	guard let msg = msg else { return }
	print(String(cString: msg))
}

enum Logger {
	static func setup() {
		register_log_callback(rustLogHandler)
	}

	static func log(_ msg: String) {
		print("[Swift] \(msg)")
	}
}
