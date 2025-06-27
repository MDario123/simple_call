package com.mdario.simplecall

import com.mdario.simplecall.ui.parseAddress
import org.junit.Test

import org.junit.Assert.*

/**
 * Example local unit test, which will execute on the development machine (host).
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
class ExampleUnitTest {
    @Test
    fun parseAddress_IsCorrect() {
        val url = "room1@127.0.0.1:8383"

        val parsed = parseAddress(url)

        assertEquals(parsed.room, "room1")
        assertEquals(parsed.ip, "127.0.0.1")
        assertEquals(parsed.port, "8383")
    }

    @Test
    fun parseAddress_IsIncorrect() {
        val url = "room1@127.0.0.18383"

        var exception: IllegalArgumentException? = null
        try {
            parseAddress(url)
        } catch (e: IllegalArgumentException) {
            exception = e
        }

        assertNotNull(exception)
    }
}