package com.autozygarde

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

class BotService : Service() {
    private var worker: Thread? = null
    private var bot: Process? = null
    private var stayOn: String? = null
    private val stopRequested = AtomicBoolean(false)

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> stopBot()
            else -> startBot()
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        stopBot()
        super.onDestroy()
    }

    private fun startBot() {
        if (worker?.isAlive == true) {
            return
        }
        stopRequested.set(false)
        BotRuntime.setRunning(true)
        val notice = notification("Startet…")
        if (Build.VERSION.SDK_INT >= 34) {
            startForeground(
                NOTIFICATION_ID,
                notice,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE,
            )
        } else {
            startForeground(NOTIFICATION_ID, notice)
        }
        worker = thread(name = "auto-zygarde") {
            try {
                runBot()
            } catch (_: InterruptedException) {
                BotRuntime.line("Stopp.")
            } catch (err: Exception) {
                BotRuntime.line(err.message ?: err.toString())
            } finally {
                restoreScreen()
                BotRuntime.setRunning(false)
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
            }
        }
    }

    private fun runBot() {
        if (stopRequested.get()) {
            return
        }
        BotRuntime.line("Prüfe Root…")
        update("Warte auf Magisk…")
        val id = runSu("id")
        if (id.code != 0 || !id.text.contains("uid=0")) {
            BotRuntime.line(id.text.ifBlank { "Root fehlt. In Magisk erlauben." })
            return
        }
        BotRuntime.line(id.text.trim())

        val launch = packageManager.getLaunchIntentForPackage(POKEMON_GO)
        if (launch == null) {
            BotRuntime.line("Pokémon GO ist nicht installiert.")
            return
        }

        if (stopRequested.get()) {
            return
        }
        prepareFiles()
        if (stopRequested.get()) {
            return
        }
        keepScreenOn()
        BotRuntime.line("Starte Pokémon GO.")
        update("Starte Pokémon GO")
        launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        startActivity(launch)
        waitForMap()

        BotRuntime.line("Starte den Bot.")
        update("Bot läuft")
        val shell = ProcessBuilder(
            "su",
            "-c",
            "cd $HOME && echo ${'$'}${'$'} > $PID_FILE && PATH=/system/bin:/system/xbin exec $BINARY routes",
        ).redirectErrorStream(true).start()
        bot = shell
        shell.inputStream.bufferedReader().useLines { lines ->
            lines.forEach { line ->
                BotRuntime.line(line)
            }
        }
        val code = shell.waitFor()
        BotRuntime.line("Bot beendet ($code).")
        update(if (code == 0) "Fertig" else "Beendet ($code)")
    }

    private fun prepareFiles() {
        BotRuntime.line("Kopiere Bot und Routen.")
        val made = runSu("mkdir -p $HOME/assets/gpx $HOME/assets/templates $HOME/data")
        if (made.code != 0) {
            error(made.text.ifBlank { "Arbeitsordner konnte nicht angelegt werden." })
        }
        copyAssetDir("gpx", "$HOME/assets/gpx")
        copyAssetDir("templates", "$HOME/assets/templates")
        assets.open("bin/auto-zygarde").use { suWrite(BINARY, it) }
        val mode = runSu("chmod 755 $BINARY")
        if (mode.code != 0) {
            error(mode.text.ifBlank { "Bot-Binary ist nicht ausführbar." })
        }
    }

    private fun copyAssetDir(assetDir: String, destDir: String) {
        val names = assets.list(assetDir) ?: emptyArray()
        if (names.isEmpty()) {
            error("Asset $assetDir fehlt in der APK.")
        }
        for (name in names) {
            assets.open("$assetDir/$name").use { suWrite("$destDir/$name", it) }
        }
    }

    private fun keepScreenOn() {
        val current = runSu("settings get global stay_on_while_plugged_in")
        if (current.code != 0) {
            return
        }
        stayOn = current.text.trim().let { if (it.isEmpty() || it == "null") "0" else it }
        runSu("svc power stayon true")
        runSu("input keyevent 224")
        BotRuntime.line("Bildschirm bleibt an.")
    }

    private fun restoreScreen() {
        val previous = stayOn ?: return
        stayOn = null
        runSu("settings put global stay_on_while_plugged_in $previous")
    }

    private fun waitForMap() {
        val deadline = System.currentTimeMillis() + 90_000
        while (System.currentTimeMillis() < deadline) {
            val front = runSu("dumpsys activity activities", 20)
            val onMap = front.text.lineSequence().any { line ->
                line.contains("com.nianticlabs.pokemongo") &&
                    (line.contains("topResumedActivity") || line.contains("mResumedActivity"))
            }
            if (onMap) {
                BotRuntime.line("Pokémon GO ist vorn. Warte auf die Karte.")
                update("Warte auf die Karte")
                Thread.sleep(20_000)
                return
            }
            Thread.sleep(1_000)
        }
        BotRuntime.line("Pokémon GO nicht als vorderste App erkannt. Der Bot startet trotzdem.")
    }

    private fun stopBot() {
        if (!stopRequested.compareAndSet(false, true)) {
            return
        }
        bot?.destroy()
        worker?.interrupt()
        thread(name = "auto-zygarde-stop", isDaemon = true) {
            runSu(
                "if [ -f $PID_FILE ]; then kill ${'$'}(cat $PID_FILE) 2>/dev/null; rm -f $PID_FILE; fi",
            )
        }
    }

    private fun update(text: String) {
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(NOTIFICATION_ID, notification(text))
    }

    private fun notification(text: String): Notification {
        val manager = getSystemService(NotificationManager::class.java)
        if (manager.getNotificationChannel(CHANNEL) == null) {
            manager.createNotificationChannel(
                NotificationChannel(CHANNEL, "Bot", NotificationManager.IMPORTANCE_LOW),
            )
        }
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val stop = PendingIntent.getService(
            this,
            1,
            Intent(this, BotService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        return Notification.Builder(this, CHANNEL)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentTitle(getString(R.string.app_name))
            .setContentText(text)
            .setContentIntent(open)
            .setOngoing(true)
            .addAction(0, getString(R.string.stop), stop)
            .build()
    }

    companion object {
        const val ACTION_START = "com.autozygarde.START"
        const val ACTION_STOP = "com.autozygarde.STOP"
        private const val CHANNEL = "bot"
        private const val NOTIFICATION_ID = 1
        private const val POKEMON_GO = "com.nianticlabs.pokemongo"
        private const val HOME = "/data/local/tmp/auto-zygarde-home"
        private const val BINARY = "/data/local/tmp/auto-zygarde"
        private const val PID_FILE = "/data/local/tmp/auto-zygarde.pid"
    }
}
