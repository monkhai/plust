package com.example.rustdroid

import android.opengl.GLSurfaceView
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import com.example.rustdroid.ui.theme.RustdroidTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Logger.setup()
        enableEdgeToEdge()
        setContent {
            RustdroidTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Box(modifier = Modifier.padding(innerPadding).fillMaxSize()) {
                        AndroidView(
                                factory = { context ->
                                    val baseUrl = "http://10.0.2.2:8080"
                                    val player = Player(baseUrl)

                                    // Create GLSurfaceView and renderer
                                    val glSurfaceView = GLSurfaceView(context).apply {
                                        setEGLContextClientVersion(2)
                                    }
                                    val renderer = VideoRenderer(context, player)

                                    // Wire up GL context for frame uploads
                                    player.setGLContext(glSurfaceView, renderer)

                                    // Set renderer and start
                                    glSurfaceView.setRenderer(renderer)
                                    glSurfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

                                    // Now safe to start decoding (GL context is set)
                                    player.startFrameDecoding()
                                    player.play()

                                    glSurfaceView
                                },
                                modifier = Modifier.fillMaxWidth().aspectRatio(16f / 9f)
                        )
                    }
                }
            }
        }
    }
}
