package com.autozygarde

import java.io.InputStream
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread

data class SuResult(val code: Int, val text: String)

fun runSu(command: String, timeoutSec: Long = 120): SuResult {
    val process = ProcessBuilder("su", "-c", command)
        .redirectErrorStream(true)
        .start()
    val output = StringBuilder()
    val reader = thread(isDaemon = true) {
        process.inputStream.bufferedReader().use { output.append(it.readText()) }
    }
    if (!process.waitFor(timeoutSec, TimeUnit.SECONDS)) {
        process.destroyForcibly()
        reader.join(1000)
        return SuResult(124, output.toString().ifBlank { "su Zeitüberschreitung" })
    }
    reader.join()
    return SuResult(process.exitValue(), output.toString())
}

fun suWrite(dest: String, input: InputStream) {
    val process = ProcessBuilder("su", "-c", "cat > '$dest'").start()
    val errors = StringBuilder()
    val reader = thread(isDaemon = true) {
        process.errorStream.bufferedReader().use { errors.append(it.readText()) }
    }
    process.outputStream.use { output -> input.copyTo(output) }
    if (!process.waitFor(120, TimeUnit.SECONDS)) {
        process.destroyForcibly()
        error("Schreiben nach $dest hat zu lange gedauert")
    }
    reader.join(1000)
    if (process.exitValue() != 0) {
        error(errors.toString().ifBlank { "Schreiben nach $dest fehlgeschlagen (${process.exitValue()})" })
    }
}
