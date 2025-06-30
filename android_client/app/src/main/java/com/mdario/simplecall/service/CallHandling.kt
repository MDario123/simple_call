package com.mdario.simplecall.service

import android.Manifest
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import android.util.Log
import androidx.annotation.RequiresPermission
import com.theeasiestway.opus.Encoder
import com.theeasiestway.opus.Decoder
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer
import kotlin.math.log10
import kotlin.math.sqrt


const val OPUS_SAMPLE_RATE = 48_000
const val OPUS_CHANNELS = 1
const val OPUS_FRAME_SIZE = 960 * 3
const val OPUS_FRAME_SIZE_BYTES = OPUS_FRAME_SIZE * Float.SIZE_BYTES
const val OPUS_APPLICATION_VOIP = 2048
const val OPUS_BITRATE = 16_000

const val SILENCE_THRESHOLD_DBFS = -50

@RequiresPermission(Manifest.permission.RECORD_AUDIO)
fun handleCall(socket: DatagramSocket, peerAddress: InetSocketAddress) {
    val captureThread = Thread {
        captureAndSend(socket, peerAddress)
    }

    val playThread = Thread {
        receiveAndPlay(socket)
    }

    captureThread.start()
    playThread.start()

    try {
        while (!Thread.interrupted()) {
            Thread.sleep(100)
        }
    } catch (_: InterruptedException) {
    } finally {
        captureThread.interrupt()
        playThread.interrupt()
        captureThread.join()
        playThread.join()
    }
}

@RequiresPermission(Manifest.permission.RECORD_AUDIO)
fun captureAndSend(socket: DatagramSocket, peerAddress: SocketAddress) {
    // Mic configuration
    val audioFormat = AudioFormat.Builder().setSampleRate(OPUS_SAMPLE_RATE)
        .setEncoding(AudioFormat.ENCODING_PCM_FLOAT).setChannelMask(AudioFormat.CHANNEL_IN_MONO)
        .build()
    val recorder: AudioRecord =
        AudioRecord.Builder().setAudioSource(MediaRecorder.AudioSource.VOICE_COMMUNICATION)
            .setAudioFormat(audioFormat)
            .build()

    recorder.startRecording()

    // Encoder configuration
    val encoder = Encoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS, OPUS_APPLICATION_VOIP)
    encoder.setBitrate(OPUS_BITRATE)

    val captureBuffer = ByteBuffer.allocateDirect(OPUS_FRAME_SIZE_BYTES)
    captureBuffer.order(ByteOrder.nativeOrder())
    val captureBufferAsFloat = captureBuffer.asFloatBuffer()

    val encodedBuffer = ByteBuffer.allocateDirect(1024)
    val sendingArray = ByteArray(1024)

    try {
        while (!Thread.interrupted()) {
            val amountRead =
                recorder.read(
                    captureBuffer,
                    OPUS_FRAME_SIZE_BYTES,
                    AudioRecord.READ_BLOCKING
                )

            if (amountRead < 0) {
                Log.e("MIC", "Read failed: $amountRead")
                continue
            }

            if (!isSilent(captureBufferAsFloat)) {
                val encoded =
                    encoder.encodeFloat(captureBufferAsFloat, OPUS_FRAME_SIZE, encodedBuffer)

                if (encoded < 0) {
                    Log.e("Encoder", "Encoding returned error: $encoded")
                    continue;
                }

                encodedBuffer.clear()
                encodedBuffer.position(encoded)
                encodedBuffer.flip()
                encodedBuffer.get(sendingArray, 0, encoded)

                socket.send(DatagramPacket(sendingArray, encoded, peerAddress))
            }
        }
    } finally {
        recorder.stop()
        recorder.release()
        encoder.release()
    }
}

fun rms(samples: FloatBuffer): Float {
    var sumOfSquares = 0.0f

    samples.position(0)
    val size = samples.remaining()

    while (samples.hasRemaining()) {
        val sample = samples.get()
        sumOfSquares += sample * sample
    }

    val mean = sumOfSquares / size
    return sqrt(mean)
}

fun dbfs(samples: FloatBuffer): Float {
    val rmsValue = rms(samples)
    if (rmsValue == 0.0f) {
        return -100.0f // Very low dBFS for silence
    }
    return 20.0f * log10(rmsValue)
}

fun isSilent(samples: FloatBuffer): Boolean {
    val vol = dbfs(samples)
    return vol < SILENCE_THRESHOLD_DBFS
}

fun receiveAndPlay(socket: DatagramSocket) {
    // Speaker configuration
    val audioAttributes =
        AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_VOICE_COMMUNICATION)
            .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build()
    val audioFormat = AudioFormat.Builder().setSampleRate(OPUS_SAMPLE_RATE)
        .setEncoding(AudioFormat.ENCODING_PCM_FLOAT).setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
        .build()
    val player =
        AudioTrack.Builder().setAudioFormat(audioFormat).setAudioAttributes(audioAttributes)
            .setTransferMode(AudioTrack.MODE_STREAM)
            .setBufferSizeInBytes(Float.SIZE_BYTES * OPUS_FRAME_SIZE * OPUS_CHANNELS * 4)
            .build()

    player.play()

    // Decoder configuration
    val decoder = Decoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS)

    val receiveArray = ByteArray(1024)
    val receivedBuffer = ByteBuffer.allocateDirect(1024)
    val decodedBuffer =
        ByteBuffer.allocateDirect(Float.SIZE_BYTES * OPUS_FRAME_SIZE * OPUS_CHANNELS)
    decodedBuffer.order(ByteOrder.nativeOrder())
    val packet = DatagramPacket(receiveArray, receiveArray.size)

    socket.soTimeout = 60

    try {
        while (!Thread.interrupted()) {
            var receivedPacket = true
            try {
                socket.receive(packet)
            } catch (e: SocketTimeoutException) {
                receivedPacket = false
            }

            var decoded = 0
            decodedBuffer.clear()

            if (receivedPacket) {
                receivedBuffer.clear()
                receivedBuffer.put(packet.data, packet.offset, packet.length)
                receivedBuffer.flip()

                decoded = decoder.decodeFloat(receivedBuffer, decodedBuffer.asFloatBuffer())
            } else {
                decoded = decoder.decodeFloat(null, decodedBuffer.asFloatBuffer())
            }

            if (decoded < 0) {
                Log.e("Decoder", "Decoding returned error: $decoded")
                continue
            }

            val enqueued = player.write(decodedBuffer, OPUS_FRAME_SIZE_BYTES, AudioTrack.WRITE_BLOCKING)

            if (enqueued != Float.SIZE_BYTES * decoded) {
                Log.e("Player", "Enqueued $enqueued bytes, expected $OPUS_FRAME_SIZE_BYTES")
            }
        }
    } finally {
        player.stop()
        player.release()
        decoder.release()
    }
}