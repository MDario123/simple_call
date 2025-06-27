package com.mdario.simplecall

import android.Manifest
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.result.contract.ActivityResultContracts.RequestMultiplePermissions
import androidx.activity.result.contract.ActivityResultContracts.RequestPermission
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.ui.Modifier
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

        // Register the permissions callback, which handles the user's response to the
        // system permissions dialog. Save the return value, an instance of
        // ActivityResultLauncher. You can use either a val, as shown in this snippet,
        // or a lateinit var in your onAttach() or onCreate() method.
        // TODO: Also ask for notification permission



        val requestPermissionLauncher = registerForActivityResult(
            RequestMultiplePermissions()
        ) { permissions ->
            if (permissions.all {it.value}) {
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
                            Column(modifier = Modifier.padding(padding)) {
                                Text(room)
                                Text(ip)
                                Text(port)
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