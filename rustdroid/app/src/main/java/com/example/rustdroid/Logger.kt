package com.example.rustdroid

import android.util.Log
import java.io.File
import java.io.FileWriter
import java.io.PrintWriter

private const val TAG = "plust"

object Logger {
    private var fileWriter: PrintWriter? = null

    private val rustCallback = object : LogCallback {
        override fun invoke(msg: String) {
            Log.d(TAG, msg)
            writeToFile("[Rust] $msg")
        }
    }

    fun setup() {
        PlustCoreLib.INSTANCE.register_log_callback(rustCallback)
        try {
            val file = File("/data/local/tmp/debug.log")
            fileWriter = PrintWriter(FileWriter(file, false))
            log("Logger initialized, writing to ${file.absolutePath}")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to open log file: ${e.message}")
        }
    }

    fun log(msg: String) {
        val fullMsg = "[Kotlin] $msg"
        Log.d(TAG, fullMsg)
        writeToFile(fullMsg)
    }

    private fun writeToFile(msg: String) {
        fileWriter?.let {
            it.println("${System.currentTimeMillis()} $msg")
            it.flush()
        }
    }
}
