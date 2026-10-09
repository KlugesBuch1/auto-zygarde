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
    private lateinit var joystick: Button
    private lateinit var toggle: Button
    private lateinit var log: TextView
    private lateinit var scroller: ScrollView
    private var joystickOpened = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        joystickOpened = savedInstanceState?.getBoolean(STATE_JOYSTICK) == true
        joystick = findViewById(R.id.joystick)
        toggle = findViewById(R.id.toggle)
        log = findViewById(R.id.log)
        scroller = findViewById(R.id.scroller)
        log.text = BotRuntime.text()
        showRunning(BotRuntime.running)
        joystick.setOnClickListener { openJoystick() }
        toggle.setOnClickListener {
            if (BotRuntime.running) {
                startService(Intent(this, BotService::class.java).setAction(BotService.ACTION_STOP))
            } else if (joystickOpened) {
                startBot()
            }
        }
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putBoolean(STATE_JOYSTICK, joystickOpened)
        super.onSaveInstanceState(outState)
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

    private fun openJoystick() {
        val launch = joystickPackage()?.let { packageManager.getLaunchIntentForPackage(it) }
        if (launch == null) {
            log.append(getString(R.string.joystick_missing))
            log.append("\n")
            scroller.post { scroller.fullScroll(ScrollView.FOCUS_DOWN) }
            return
        }
        joystickOpened = true
        showRunning(BotRuntime.running)
        startActivity(launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }

    private fun joystickPackage(): String? {
        val probe = Intent(JOYSTICK_TELEPORT)
        val services = if (Build.VERSION.SDK_INT >= 33) {
            packageManager.queryIntentServices(probe, PackageManager.ResolveInfoFlags.of(0))
        } else {
            @Suppress("DEPRECATION")
            packageManager.queryIntentServices(probe, 0)
        }
        services.firstOrNull()?.serviceInfo?.packageName?.let { return it }
        return JOYSTICK_PACKAGE.takeIf { packageManager.getLaunchIntentForPackage(it) != null }
    }

    private fun showRunning(running: Boolean) {
        toggle.setText(if (running) R.string.stop else R.string.start)
        toggle.isEnabled = running || joystickOpened
    }

    companion object {
        private const val STATE_JOYSTICK = "joystick_opened"
        private const val JOYSTICK_PACKAGE = "com.theappninjas.gpsjoystick"
        private const val JOYSTICK_TELEPORT = "theappninjas.gpsjoystick.TELEPORT"
    }
}
