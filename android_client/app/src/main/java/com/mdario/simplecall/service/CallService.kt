package com.mdario.simplecall.service

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import androidx.annotation.RequiresPermission
import androidx.core.app.NotificationCompat
import com.mdario.simplecall.R
import java.net.InetAddress



class CallService : Service() {
    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    private var callThreadHandler: Thread? = null

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null) {
            Log.e("CallService", "Expected intent not null.")
            return START_NOT_STICKY
        }
        val intent = intent

        callThreadHandler?.interrupt()
        callThreadHandler?.join()
        callThreadHandler = null

        if (intent.hasExtra("stop")) {
            stopSelf(startId)
            Log.i("CallService", "Stopping call.")
            return START_NOT_STICKY
        }

        startForeground(startId)

        val room = intent.getStringExtra("room")
        val ip = intent.getStringExtra("ip")
        val port = intent.getStringExtra("port")

        callThreadHandler = Thread {
            handleCoordination(
                InetAddress.getByName(ip),
                port!!.toInt(),
                room!!,
                true,
            )
        }

        callThreadHandler?.start()

        return START_STICKY
    }

    private fun startForeground(startId: Int) {
        val channelID = "CHANNEL_ID"
        val channelName = "CHANNEL_NAME"

        // Create notification channel
        val channel = NotificationChannel(
            channelID,
            channelName,
            NotificationManager.IMPORTANCE_HIGH
        )
        val notificationManager = getSystemService<NotificationManager?>(NotificationManager::class.java)
        notificationManager.createNotificationChannel(channel)

        // Configure intent for the notification action
        var intent = Intent(this, CallService::class.java)
        intent.putExtra("stop", true)
        var pendingIntent = PendingIntent.getForegroundService(this, 0, intent, PendingIntent.FLAG_MUTABLE)

        // Configure notification
        val notification = NotificationCompat.Builder(this, channelID)
            .setContentTitle("Calling")
            .setContentText("Currently in room: XXXX")
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .addAction(R.drawable.ic_launcher_foreground, "Stop", pendingIntent)
            .build()

        // Finally call startForeground
        startForeground(startId, notification)
    }
}
