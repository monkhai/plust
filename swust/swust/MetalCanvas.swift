import CoreMedia
import MetalKit
import SwiftUI

struct MetalCanvas: UIViewRepresentable {
    var player: Player?

    func makeCoordinator() -> MetalRenderer { MetalRenderer() }

    func makeUIView(context: Context) -> UIView {
        guard let device = MTLCreateSystemDefaultDevice() else {
            return fallback("No Metal device (Preview/Simulator)")
        }

        let v = MTKView(frame: .zero, device: device)
        v.framebufferOnly = true
        v.clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 1)

        v.delegate = context.coordinator
        context.coordinator.setup(device: device, view: v)
        context.coordinator.player = self.player

        return v
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.player = self.player
    }

    private func fallback(_ text: String) -> UIView {
        let label = UILabel()
        label.text = text
        label.numberOfLines = 0
        label.textAlignment = .center
        return label
    }
}
