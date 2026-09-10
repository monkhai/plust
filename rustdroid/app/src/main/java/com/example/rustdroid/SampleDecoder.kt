package com.example.rustdroid

import android.media.MediaCodec
import android.media.MediaFormat
import java.io.Closeable
import java.nio.ByteBuffer

class SampleDecoderException(message: String) : Exception(message) {}

private val ANNEX_B_START_CODE = byteArrayOf(0x00, 0x00, 0x00, 0x01)

private fun avcToAnnexB(data: ByteArray, nalLengthSize: Int): ByteArray {
  // Pass 1: calculate total output size
  var totalSize = 0
  var offset = 0
  while (offset + nalLengthSize <= data.size) {
    var nalLen = 0
    for (i in 0 until nalLengthSize) {
      nalLen = (nalLen shl 8) or (data[offset + i].toInt() and 0xFF)
    }
    if (nalLen <= 0 || offset + nalLengthSize + nalLen > data.size) break
    totalSize += 4 + nalLen // start code + NAL data
    offset += nalLengthSize + nalLen
  }

  // Pass 2: allocate and copy
  val result = ByteArray(totalSize)
  var readOffset = 0
  var writeOffset = 0
  while (readOffset + nalLengthSize <= data.size) {
    var nalLen = 0
    for (i in 0 until nalLengthSize) {
      nalLen = (nalLen shl 8) or (data[readOffset + i].toInt() and 0xFF)
    }
    if (nalLen <= 0 || readOffset + nalLengthSize + nalLen > data.size) break

    // Copy start code
    System.arraycopy(ANNEX_B_START_CODE, 0, result, writeOffset, 4)
    writeOffset += 4

    // Copy NAL data
    System.arraycopy(data, readOffset + nalLengthSize, result, writeOffset, nalLen)
    writeOffset += nalLen

    readOffset += nalLengthSize + nalLen
  }
  return result
}

class SampleDecoder(baseUrl: String) : Closeable {
  private var codec: MediaCodec? = null
  private var nalLengthSize = 4
  private var config: CodecConfig

  init {
    val config =
            PlustCore.getCodecConfig(baseUrl)
                    ?: throw SampleDecoderException("failed to get config")
    this.config = config
    this.nalLengthSize = config.nalLengthSize

    val mediaFormat =
            MediaFormat.createVideoFormat(
                    MediaFormat.MIMETYPE_VIDEO_AVC,
                    config.width,
                    config.height
            )
    val annexBPrefix = byteArrayOf(0x00, 0x00, 0x00, 0x01)
    mediaFormat.setByteBuffer("csd-0", ByteBuffer.wrap(annexBPrefix + config.sps))
    mediaFormat.setByteBuffer("csd-1", ByteBuffer.wrap(annexBPrefix + config.pps))
    mediaFormat.setInteger(MediaFormat.KEY_COLOR_STANDARD, MediaFormat.COLOR_STANDARD_BT709)
    mediaFormat.setInteger(MediaFormat.KEY_COLOR_RANGE, MediaFormat.COLOR_RANGE_LIMITED)
    mediaFormat.setInteger(MediaFormat.KEY_COLOR_TRANSFER, MediaFormat.COLOR_TRANSFER_SDR_VIDEO)

    val codec = MediaCodec.createDecoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
    codec.configure(mediaFormat, null, null, 0)
    codec.start()
    this.codec = codec
  }

  fun decode(sample: Sample): DecodedFrame? {
    val decoder = this.codec ?: throw SampleDecoderException("no codec")

    if (!queueInputSample(decoder, sample)) return null

    val bufferInfo = MediaCodec.BufferInfo()
    val oBufferIndex = decoder.dequeueOutputBuffer(bufferInfo, 1000)
    if (oBufferIndex < 0) return null

    val image = decoder.getOutputImage(oBufferIndex) ?: return null
    val frame = try {
      val yPlane = copyYPlane(image)
      val uvPlane = copyUvPlane(image)
      val pts = bufferInfo.presentationTimeUs / 1_000_000.0
      DecodedFrame(config.width, config.height, yPlane, uvPlane, pts)
    } finally {
      image.close()
      decoder.releaseOutputBuffer(oBufferIndex, false)
    }

    return frame
  }

  private fun queueInputSample(decoder: MediaCodec, sample: Sample): Boolean {
    val iBufferIndex = decoder.dequeueInputBuffer(10000)
    if (iBufferIndex < 0) return false

    val iBuffer = decoder.getInputBuffer(iBufferIndex) ?: return false
    iBuffer.clear()

    val sampleData = avcToAnnexB(sample.data, this.nalLengthSize)
    iBuffer.put(sampleData)

    val flag = if (sample.isKey) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0
    val ptsUs = (sample.pts * 1_000_000).toLong()
    decoder.queueInputBuffer(iBufferIndex, 0, sampleData.size, ptsUs, flag)

    return true
  }

  private fun copyYPlane(image: android.media.Image): ByteBuffer {
    val plane = image.planes[0]
    val buffer = plane.buffer  // Cache buffer - accessing property repeatedly may return different instances
    val rowStride = plane.rowStride
    val width = this.config.width
    val height = this.config.height

    val yPlane = ByteBuffer.allocateDirect(width * height)
    val rowBytes = ByteArray(width)

    for (row in 0 until height) {
      buffer.position(row * rowStride)
      buffer.get(rowBytes, 0, width)
      yPlane.put(rowBytes)
    }

    yPlane.rewind()
    return yPlane
  }

  private fun copyUvPlane(image: android.media.Image): ByteBuffer {
    val uPlane = image.planes[1]
    val vPlane = image.planes[2]
    val width = this.config.width
    val height = this.config.height
    val uvHeight = height / 2
    val uvWidth = width / 2

    val uvOutputWidth = width
    val uvBuffer = ByteBuffer.allocateDirect(uvOutputWidth * uvHeight)

    if (uPlane.pixelStride == 2) {
      copyInterleavedUv(uPlane, uvBuffer, uvHeight, uvOutputWidth)
    } else {
      interleavePlanarUv(uPlane, vPlane, uvBuffer, uvHeight, uvWidth)
    }

    uvBuffer.rewind()
    return uvBuffer
  }

  private fun copyInterleavedUv(
    plane: android.media.Image.Plane,
    output: ByteBuffer,
    uvHeight: Int,
    uvOutputWidth: Int
  ) {
    val buffer = plane.buffer  // Cache buffer
    val rowStride = plane.rowStride
    val rowBytes = ByteArray(uvOutputWidth)
    for (row in 0 until uvHeight) {
      buffer.position(row * rowStride)
      buffer.get(rowBytes, 0, uvOutputWidth)
      output.put(rowBytes)
    }
  }

  private fun interleavePlanarUv(
    uPlane: android.media.Image.Plane,
    vPlane: android.media.Image.Plane,
    output: ByteBuffer,
    uvHeight: Int,
    uvWidth: Int
  ) {
    val uBuffer = uPlane.buffer  // Cache buffers
    val vBuffer = vPlane.buffer
    val uRowStride = uPlane.rowStride
    val vRowStride = vPlane.rowStride
    for (row in 0 until uvHeight) {
      uBuffer.position(row * uRowStride)
      vBuffer.position(row * vRowStride)
      for (col in 0 until uvWidth) {
        output.put(uBuffer.get())
        output.put(vBuffer.get())
      }
    }
  }

  override fun close() {
    codec?.stop()
    codec?.release()
    codec = null
  }
}
