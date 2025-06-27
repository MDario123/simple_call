package com.mdario.simplecall.service

import android.Manifest
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import android.util.Log
import androidx.annotation.RequiresPermission
import com.theeasiestway.opus.Constants
import com.theeasiestway.opus.Opus
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.math.log10
import kotlin.math.sqrt


const val OPUS_SAMPLE_RATE = 48000
const val OPUS_CHANNELS = 1
const val OPUS_FRAME_SIZE = 960 * 3
const val SILENCE_THRESHOLD_DBFS = -60

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
    }
}

@RequiresPermission(Manifest.permission.RECORD_AUDIO)
fun captureAndSend(socket: DatagramSocket, peerAddress: SocketAddress) {
    // Mic configuration
    val audioFormat = AudioFormat.Builder().setSampleRate(OPUS_SAMPLE_RATE)
        .setEncoding(AudioFormat.ENCODING_PCM_16BIT).setChannelMask(AudioFormat.CHANNEL_IN_MONO)
        .build()
    val recorder: AudioRecord =
        AudioRecord.Builder().setAudioSource(MediaRecorder.AudioSource.VOICE_COMMUNICATION)
            .setAudioFormat(audioFormat)
            .build()

    recorder.startRecording()

    // Encoder configuration
    val codec = Opus()
    codec.encoderInit(
        Constants.SampleRate._48000(), Constants.Channels.mono(), Constants.Application.voip()
    )
    codec.encoderSetBitrate(Constants.Bitrate.instance(16_000))

    val captureBuffer = ShortArray(OPUS_FRAME_SIZE)
    val byteBuffer = ByteBuffer.allocate(OPUS_FRAME_SIZE * 2)
    var bufferFilled = 0

    try {
        while (!Thread.interrupted()) {
            val amountRead =
                recorder.read(captureBuffer, bufferFilled, OPUS_FRAME_SIZE - bufferFilled)

            if (amountRead < 0) {
                Log.e("MIC", "Read failed: $amountRead")
                continue
            }
            bufferFilled += amountRead

            if (bufferFilled < OPUS_FRAME_SIZE) {
                continue
            }
            bufferFilled = 0

            if (!isSilent(captureBuffer)) {
                // Convert ShortArray to ByteBuffer
                byteBuffer.clear()
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                for (i in 0 until OPUS_FRAME_SIZE) {
                    byteBuffer.putShort(captureBuffer[i])
                }
                byteBuffer.flip()

                val encoded =
                    codec.encode(byteBuffer.array(), Constants.FrameSize._custom(OPUS_FRAME_SIZE))!!

                socket.send(DatagramPacket(encoded, encoded.size, peerAddress))
            }
        }
    } finally {
        recorder.stop()
        recorder.release()
        codec.encoderRelease()
    }
}

fun rms(samples: ShortArray): Float {
    if (samples.isEmpty()) return 0.0f

    val sumOfSquares = samples.map { sample ->
        val norm = sample.toFloat() / Short.MAX_VALUE
        norm * norm
    }.sum()

    val mean = sumOfSquares / samples.size
    return sqrt(mean)
}

fun dbfs(samples: ShortArray): Float {
    val rmsValue = rms(samples)
    if (rmsValue == 0.0f) {
        return -100.0f // Very low dBFS for silence
    }
    return 20.0f * log10(rmsValue)
}

fun isSilent(samples: ShortArray): Boolean {
    val vol = dbfs(samples)
    Log.d("Volume", "$vol")
    return vol < SILENCE_THRESHOLD_DBFS
}

fun receiveAndPlay(socket: DatagramSocket) {
    // Speaker configuration
    val audioAttributes =
        AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_VOICE_COMMUNICATION)
            .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build()
    val audioFormat = AudioFormat.Builder().setSampleRate(OPUS_SAMPLE_RATE)
        .setEncoding(AudioFormat.ENCODING_PCM_16BIT).setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
        .build()
    val player =
        AudioTrack.Builder().setAudioFormat(audioFormat).setAudioAttributes(audioAttributes)
            .setTransferMode(AudioTrack.MODE_STREAM).setBufferSizeInBytes(OPUS_FRAME_SIZE * 2)
            .build()

    player.play()

    // Decoder configuration
    val codec = Opus()
    codec.decoderInit(Constants.SampleRate._48000(), Constants.Channels.mono())

    val receiveBuffer = ByteArray(4096)
    val packet = DatagramPacket(receiveBuffer, receiveBuffer.size)
    val playerBuffer = ShortArray(OPUS_FRAME_SIZE)

    socket.soTimeout = 60

    try {
        while (!Thread.interrupted()) {
            var decoded: ByteArray? = null
            try {
                socket.receive(packet)
            } catch (e: SocketTimeoutException) {
                decoded = codec.decode(
                    ByteArray(0), Constants.FrameSize._custom(OPUS_FRAME_SIZE)
                )
            }

            if (decoded == null) {
                decoded = codec.decode(
                    packet.data.sliceArray(packet.offset..(packet.offset + packet.length)),
                    Constants.FrameSize._custom(OPUS_FRAME_SIZE)
                )
            }

            if (decoded == null) {
                continue
            }

            decoded.getAsShortsLE(playerBuffer)

            player.write(playerBuffer, 0, OPUS_FRAME_SIZE)
        }
    } finally {
        player.stop()
        player.release()
        codec.decoderRelease()
    }
}

fun ByteArray.getAsShortsLE(dest: ShortArray) {
    require(size % 2 == 0) { "Byte array length must be even." }
    require(dest.size >= size / 2) { "ShortArray does not have enough space." }

    for (i in 0 until size step 2) {
        val low = this[i].toInt() and 0xFF
        val high = this[i + 1].toInt() and 0xFF
        val value = ((high shl 8) or low).toShort()
        dest[i / 2] = value
    }
}