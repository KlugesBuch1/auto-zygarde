package com.autozygarde

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.widget.Button
import android.widget.ScrollView
import android.widget.TextView

class MainActivity : Activity() {
    private lateinit var toggle: Button
    private lateinit var log: TextView
    private lateinit var scroller: ScrollView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        toggle = findViewById(R.id.toggle)
        log = findViewById(R.id.log)
        scroller = findViewById(R.id.scroller)
        log.text = BotRuntime.text()
        showRunning(BotRuntime.running)
        toggle.setOnClickListener {
            if (BotRuntime.running) {
                startService(Intent(this, BotService::class.java).setAction(BotService.ACTION_STOP))
            } else {
                startBot()
            }
        }
    }

    override fun onStart() {
        super.onStart()
        BotRuntime.onLine = { line ->
            log.append(line)
            log.append("\n")
            scroller.post { scroller.fullScroll(ScrollView.FOCUS_DOWN) }
        }
        BotRuntime.onRunning = { running -> showRunning(running) }
        log.text = BotRuntime.text()
        showRunning(BotRuntime.running)
    }

    override fun onStop() {
        BotRuntime.onLine = null
        BotRuntime.onRunning = null
        super.onStop()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == 1) {
            launchService()
        }
    }

    private fun startBot() {
        if (Build.VERSION.SDK_INT >= 33 &&
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
            return
        }
        launchService()
    }

    private fun launchService() {
        val intent = Intent(this, BotService::class.java).setAction(BotService.ACTION_START)
        if (Build.VERSION.SDK_INT >= 26) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun showRunning(running: Boolean) {
        toggle.setText(if (running) R.string.stop else R.string.start)
    }
}
