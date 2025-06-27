package com.mdario.simplecall.service

import android.Manifest
import androidx.annotation.RequiresPermission
import java.io.InputStream
import java.io.OutputStream
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.security.MessageDigest

const val SIGNAL_WAITING_IN_ROOM: Byte = 1
const val SIGNAL_PARTNER_FOUND: Byte = 2
const val SIGNAL_READY: Byte = 3

@RequiresPermission(Manifest.permission.RECORD_AUDIO)
fun handleCoordination(
    host: InetAddress,
    hostTcpPort: Int,
    room: String,
    relay: Boolean
) {
    try {
        val tcpSocket = Socket(host, hostTcpPort)
        tcpSocket.soTimeout = 10
        val output: OutputStream = tcpSocket.getOutputStream()
        val input: InputStream = tcpSocket.getInputStream()

        // Send SHA-512 hash of room name
        val digest = MessageDigest.getInstance("SHA-512").digest(room.toByteArray())
        output.write(digest)

        // Send relay flag
        output.write(if (relay) 1 else 0)

        val buffer = ByteArray(1024)
        var patience = 100

        val serverUdpPort: Int = run loop@{
            while (true) {
//                Thread.sleep(10)

                var size = 0
                try {
                    size = input.read(buffer, 0, 1)
                } catch (_: java.net.SocketTimeoutException) {}
                if (size == 1) {
                    when (buffer[0]) {
                        SIGNAL_WAITING_IN_ROOM -> {
                            patience = 6000
                        }

                        SIGNAL_PARTNER_FOUND -> {
                            input.read(buffer, 0, 2)
                            return@loop ((buffer[0].toInt() and 0xFF) shl 8) or (buffer[1].toInt() and 0xFF)
                        }

                        SIGNAL_READY -> {
                            output.write(byteArrayOf(SIGNAL_READY))
                        }

                        else -> throw RuntimeException("Unexpected signal from server: ${buffer[0]}")
                    }
                } else {
                    patience--
                }

                if (patience == 0) {
                    throw RuntimeException("Server is not responding.")
                }
            }

            // Needed to satisfy return type if loop doesn't return early
            @Suppress("UNREACHABLE_CODE")
            -1
        }

        val udpSocket = DatagramSocket()

        val serverUdpAddr = InetSocketAddress(host, serverUdpPort)

        // Send empty packet to the server
        udpSocket.send(DatagramPacket(ByteArray(0), 0, serverUdpAddr))

        val peerUdpAddr: InetSocketAddress = if (relay) {
            serverUdpAddr
        } else {
            val readBytes = input.read(buffer)
            if (readBytes != 6) {
                throw RuntimeException("Expected 6 bytes (IPv4 + port), but got $readBytes")
            }

            addrFromBytes(buffer.copyOfRange(0, 6))
        }

        handleCall(udpSocket, peerUdpAddr)
    } catch (_: InterruptedException) {
        return
    }
}

fun addrFromBytes(buffer: ByteArray): InetSocketAddress {
    require(buffer.size >= 6) { "Expected at least 6 bytes, got ${buffer.size}" }

    val ipBytes = buffer.copyOfRange(0, 4)
    val portBytes = buffer.copyOfRange(4, 6)

    val ip = InetAddress.getByAddress(ipBytes)
    val port = ((portBytes[0].toInt() and 0xFF) shl 8) or (portBytes[1].toInt() and 0xFF)

    return InetSocketAddress(ip, port)
}