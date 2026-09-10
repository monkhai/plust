import CoreMedia
import MetalKit

final class MetalRenderer: NSObject, MTKViewDelegate {
    var player: Player?

    var encoder: MTLCommandEncoder? = nil

    private var startTime: CFTimeInterval? = nil

    private var queue: MTLCommandQueue?
    private var pipeline: MTLRenderPipelineState?

    func setup(device: MTLDevice, view: MTKView) {
        queue = device.makeCommandQueue()
        let library = device.makeDefaultLibrary()!

        let togglePipelineDesc = MTLRenderPipelineDescriptor()
        togglePipelineDesc.vertexFunction = library.makeFunction(name: "vertex_main")
        togglePipelineDesc.fragmentFunction = library.makeFunction(name: "yuv_fragment")
        togglePipelineDesc.colorAttachments[0].pixelFormat = view.colorPixelFormat
        pipeline = try? device.makeRenderPipelineState(descriptor: togglePipelineDesc)
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard
            let queue,
            let descriptor = view.currentRenderPassDescriptor,
            let drawable = view.currentDrawable,
            let commandBuffer = queue.makeCommandBuffer(),
            let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor),
            let pipeline = pipeline,
            let player = self.player
        else { return }

        encoder.setRenderPipelineState(pipeline)
        if let frame = player.getCurrentFrame() {
            if let y = frame.yTexture, let uv = frame.uvTexture {
                encoder.setFragmentTexture(y, index: 0)
                encoder.setFragmentTexture(uv, index: 1)
            }
        }

        encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)
        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

}
