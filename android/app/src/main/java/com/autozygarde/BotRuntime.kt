package com.autozygarde

import android.os.Handler
import android.os.Looper

object BotRuntime {
    @Volatile
    var running: Boolean = false
        private set

    private val log = StringBuilder()
    var onLine: ((String) -> Unit)? = null
    var onRunning: ((Boolean) -> Unit)? = null

    fun text(): String = synchronized(log) { log.toString() }

    fun line(message: String) {
        synchronized(log) {
            log.append(message).append('\n')
        }
        val callback = onLine ?: return
        Handler(Looper.getMainLooper()).post { callback(message) }
    }

    fun setRunning(value: Boolean) {
        running = value
        val callback = onRunning ?: return
        Handler(Looper.getMainLooper()).post { callback(value) }
    }
}
