package com.example.rustdroid

import android.content.Context
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class VideoRenderer(private val context: Context, private val player: Player) : GLSurfaceView.Renderer {

    // ==================== State ====================
    private var program: Int = 0

    private var aPositionLoc: Int = 0
    private var aTexCoordLoc: Int = 0
    private var yTextureLoc: Int = 0
    private var uvTextureLoc: Int = 0

    private lateinit var vertexBuffer: FloatBuffer

    // GPU texture pool - initialized lazily when dimensions are known
    private var texturePool: TexturePool? = null
    private var poolWidth: Int = 0
    private var poolHeight: Int = 0

    // Current frame to render
    private var currentFrame: GpuFrame? = null

    // Fullscreen quad vertices: x, y, u, v
    private val vertices =
            floatArrayOf(
                    -1.0f, -1.0f, 0.0f, 1.0f, // bottom-left
                    1.0f, -1.0f, 1.0f, 1.0f,  // bottom-right
                    -1.0f, 1.0f, 0.0f, 0.0f,  // top-left
                    1.0f, 1.0f, 1.0f, 0.0f,   // top-right
            )

    // ==================== GLSurfaceView.Renderer ====================

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        GLES20.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)

        val success = createProgram()
        if (!success) return
        getLocations()
        createVertexBuffer()
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        GLES20.glViewport(0, 0, width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)

        // Get current frame from player
        val frame = player.getCurrentFrame()
        if (frame == null) return

        currentFrame = frame

        GLES20.glUseProgram(program)
        bindTextures(frame)
        bindVertexAttributes()
        drawQuad()
    }

    // ==================== GPU Texture Management ====================

    /**
     * Get the texture pool, initializing it if needed with the given dimensions.
     * Must be called on GL thread.
     */
    fun getOrCreateTexturePool(width: Int, height: Int): TexturePool {
        val pool = texturePool
        if (pool != null && poolWidth == width && poolHeight == height) {
            return pool
        }

        // Destroy old pool if dimensions changed
        texturePool?.destroy()

        // Create new pool - 90 frames = 3 seconds at 30fps
        val newPool = TexturePool(90, width, height)
        newPool.initialize()
        texturePool = newPool
        poolWidth = width
        poolHeight = height
        return newPool
    }

    /**
     * Upload a decoded frame to GPU textures and return the GpuFrame.
     * Must be called on GL thread.
     */
    fun uploadFrame(decodedFrame: DecodedFrame): GpuFrame? {
        val pool = getOrCreateTexturePool(decodedFrame.width, decodedFrame.height)
        val slot = pool.acquire() ?: return null

        pool.uploadFrame(slot, decodedFrame.yPlane, decodedFrame.uvPlane)

        return GpuFrame(
            width = decodedFrame.width,
            height = decodedFrame.height,
            yTextureId = pool.getYTextureId(slot),
            uvTextureId = pool.getUVTextureId(slot),
            pts = decodedFrame.pts,
            poolSlot = slot
        )
    }

    /**
     * Get the texture pool. May be null if not initialized yet.
     */
    fun getTexturePool(): TexturePool? = texturePool

    // ==================== Setup Helpers ====================

    private fun createProgram(): Boolean {
        val vertexShader =
                compileShader(GLES20.GL_VERTEX_SHADER, loadRawResource(R.raw.vertex_shader))
        val fragmentShader =
                compileShader(GLES20.GL_FRAGMENT_SHADER, loadRawResource(R.raw.fragment_shader))

        program = GLES20.glCreateProgram()
        GLES20.glAttachShader(program, vertexShader)
        GLES20.glAttachShader(program, fragmentShader)
        GLES20.glLinkProgram(program)

        val linkStatus = IntArray(1)
        GLES20.glGetProgramiv(program, GLES20.GL_LINK_STATUS, linkStatus, 0)
        if (linkStatus[0] == 0) {
            GLES20.glDeleteProgram(program)
            return false
        }
        return true
    }

    private fun getLocations() {
        aPositionLoc = GLES20.glGetAttribLocation(program, "aPosition")
        aTexCoordLoc = GLES20.glGetAttribLocation(program, "aTexCoord")
        yTextureLoc = GLES20.glGetUniformLocation(program, "yTexture")
        uvTextureLoc = GLES20.glGetUniformLocation(program, "uvTexture")
    }

    private fun createVertexBuffer() {
        vertexBuffer =
                ByteBuffer.allocateDirect(vertices.size * 4)
                        .order(ByteOrder.nativeOrder())
                        .asFloatBuffer()
                        .put(vertices)
        vertexBuffer.position(0)
    }

    // ==================== Render Helpers ====================

    /**
     * Bind existing GPU textures for rendering (fast - no upload).
     */
    private fun bindTextures(frame: GpuFrame) {
        GLES20.glActiveTexture(GLES20.GL_TEXTURE0)
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, frame.yTextureId)
        GLES20.glUniform1i(yTextureLoc, 0)

        GLES20.glActiveTexture(GLES20.GL_TEXTURE1)
        GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, frame.uvTextureId)
        GLES20.glUniform1i(uvTextureLoc, 1)
    }

    private fun bindVertexAttributes() {
        vertexBuffer.position(0)
        GLES20.glVertexAttribPointer(aPositionLoc, 2, GLES20.GL_FLOAT, false, 16, vertexBuffer)
        GLES20.glEnableVertexAttribArray(aPositionLoc)

        vertexBuffer.position(2)
        GLES20.glVertexAttribPointer(aTexCoordLoc, 2, GLES20.GL_FLOAT, false, 16, vertexBuffer)
        GLES20.glEnableVertexAttribArray(aTexCoordLoc)
    }

    private fun drawQuad() {
        GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)
        GLES20.glDisableVertexAttribArray(aPositionLoc)
        GLES20.glDisableVertexAttribArray(aTexCoordLoc)
    }

    // ==================== Shader Helpers ====================

    private fun loadRawResource(resourceId: Int): String {
        return context.resources.openRawResource(resourceId).bufferedReader().readText()
    }

    private fun compileShader(type: Int, code: String): Int {
        val shader = GLES20.glCreateShader(type)
        GLES20.glShaderSource(shader, code)
        GLES20.glCompileShader(shader)

        val compileStatus = IntArray(1)
        GLES20.glGetShaderiv(shader, GLES20.GL_COMPILE_STATUS, compileStatus, 0)
        if (compileStatus[0] == 0) {
            GLES20.glDeleteShader(shader)
            return 0
        }
        return shader
    }
}
