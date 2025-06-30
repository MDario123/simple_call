package com.mdario.simplecall

import android.Manifest
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.result.contract.ActivityResultContracts.RequestMultiplePermissions
import androidx.activity.result.contract.ActivityResultContracts.RequestPermission
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.mdario.simplecall.service.CallService
import com.mdario.simplecall.ui.theme.SimpleCallTheme

class CallActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val room = intent.getStringExtra("room")
        val ip = intent.getStringExtra("ip")
        val port = intent.getStringExtra("port")

        if (room == null || ip == null || port == null) {
            throw Error("This should never happen.")
        }

        val requestPermissionLauncher = registerForActivityResult(
            RequestMultiplePermissions()
        ) { permissions ->
            if (permissions.all { it.value }) {
                var callIntent = Intent(this, CallService::class.java)
                callIntent.putExtra("room", room)
                callIntent.putExtra("ip", ip)
                callIntent.putExtra("port", port)

                startForegroundService(callIntent)

                setContent {
                    SimpleCallTheme {
                        Scaffold(
                            topBar = {
                                TopAppBar(
                                    title = {
                                        Text("SimpleCall")
                                    })
                            }) { padding ->
                            Column(
                                modifier = Modifier
                                    .padding(padding)
                                    .fillMaxSize(),
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.Center
                            ) {
                                Text(room, fontWeight = FontWeight.Bold, fontSize = 60.sp)
                                Text("$ip:$port", fontSize = 40.sp)
                            }
                        }
                    }
                }
            } else {
                finish()
            }
        }

        requestPermissionLauncher.launch(
            arrayOf(Manifest.permission.RECORD_AUDIO, Manifest.permission.POST_NOTIFICATIONS)
        )

    }
}