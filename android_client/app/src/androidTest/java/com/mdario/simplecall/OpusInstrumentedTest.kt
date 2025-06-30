package com.mdario.simplecall

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.theeasiestway.opus.Decoder
import com.theeasiestway.opus.Encoder

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*
import java.nio.ByteBuffer
import java.nio.ByteOrder

const val OPUS_SAMPLE_RATE = 48_000
const val OPUS_CHANNELS = 1
const val OPUS_FRAME_SIZE = 960 * 3
const val OPUS_APPLICATION_VOIP = 2048
const val OPUS_BITRATE = 16_000

@RunWith(AndroidJUnit4::class)
class OpusInstrumentedTest {
    @Test
    fun createEncoder() {
        val encoder = Encoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS, OPUS_APPLICATION_VOIP)
        assertNotNull(encoder)
        assert(encoder.setBitrate(OPUS_BITRATE) == 0)
        encoder.release()
    }

    @Test
    fun encodeFloat() {
        val encoder = Encoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS, OPUS_APPLICATION_VOIP)
        assertNotNull(encoder)
        assert(encoder.setBitrate(OPUS_BITRATE) == 0)

        val inputBuffer = ByteBuffer.allocateDirect(Float.SIZE_BYTES * OPUS_FRAME_SIZE * OPUS_CHANNELS)
        inputBuffer.order(ByteOrder.nativeOrder())
        val inputBufferAsFloat = inputBuffer.asFloatBuffer()

        val encodedBuffer = ByteBuffer.allocateDirect(1024)
        encodedBuffer.order(ByteOrder.nativeOrder())

        for (i in 0 until inputBufferAsFloat.remaining()) {
            inputBufferAsFloat.put(0.5f)
        }

        val amountEncoded = encoder.encodeFloat(inputBuffer.asFloatBuffer(), OPUS_FRAME_SIZE, encodedBuffer)

        assert(amountEncoded > 0) {"Error ocurred: $amountEncoded"}

        encoder.release()
    }

    @Test
    fun createDecoder() {
        val decoder = Decoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS)
        assertNotNull(decoder)
        decoder.release()
    }

    @Test
    fun decodeFloat() {
        // Encode
        val encoder = Encoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS, OPUS_APPLICATION_VOIP)
        assertNotNull(encoder)

        val inputBuffer = ByteBuffer.allocateDirect(Float.SIZE_BYTES * OPUS_FRAME_SIZE * OPUS_CHANNELS)
        inputBuffer.order(ByteOrder.nativeOrder())
        val inputBufferAsFloat = inputBuffer.asFloatBuffer()

        val encodedBuffer = ByteBuffer.allocateDirect(1024)
        encodedBuffer.order(ByteOrder.nativeOrder())

        for (i in 0 until inputBufferAsFloat.remaining()) {
            inputBufferAsFloat.put(0.5f)
        }

        val amountEncoded = encoder.encodeFloat(inputBufferAsFloat, OPUS_FRAME_SIZE, encodedBuffer)

        assert(amountEncoded > 0) {"Error ocurred: $amountEncoded"}

        encoder.release()

        // Decode
        val decoder = Decoder(OPUS_SAMPLE_RATE, OPUS_CHANNELS)
        assertNotNull(decoder)

        val decoded = decoder.decodeFloat(encodedBuffer, inputBufferAsFloat)

        assertEquals(decoded, OPUS_FRAME_SIZE)

        decoder.release()
    }
}