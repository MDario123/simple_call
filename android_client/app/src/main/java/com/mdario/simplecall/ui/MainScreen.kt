package com.mdario.simplecall.ui

import android.content.Intent
import android.widget.Toast
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.mdario.simplecall.R
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat.startActivity
import com.mdario.simplecall.CallActivity
import com.mdario.simplecall.MainActivity
import com.mdario.simplecall.data.MainViewModel

@Composable
fun MainScreen(context: MainActivity, mainViewModel: MainViewModel, suggestedUrls: List<String>) {
    var search = rememberSaveable { mutableStateOf("") }

    LazyColumn(
        contentPadding = PaddingValues(16.dp),
    ) {
        item {
            OutlinedTextField(
                modifier = Modifier.fillMaxWidth(),
                value = search.value,
                singleLine = true,
                onValueChange = {
                    search.value = it
                },
                label = {
                    Text(stringResource(R.string.connection_string_hint))
                },
                keyboardOptions = KeyboardOptions.Default.copy(
                    imeAction = ImeAction.Go
                ),
                keyboardActions = KeyboardActions(
                    onGo = {
                        try {
                            val address = parseAddress(search.value)
                            mainViewModel.addUrl(search.value)

                            var intent = Intent(context, CallActivity::class.java)
                            intent.putExtra("room", address.room)
                            intent.putExtra("ip", address.ip)
                            intent.putExtra("port", address.port)

                            startActivity(context, intent, null)
                        } catch (e: IllegalArgumentException) {
                            Toast.makeText(context, e.message, Toast.LENGTH_SHORT).show()
                        }
                    }
                )
            )
        }

        items(suggestedUrls) { url ->
            Button(
                onClick = { search.value = url },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(8.dp)
            ) {
                Text(url)
            }
        }
    }
}


val cardinals = listOf("first", "second", "third", "fourth")

/**
 * @throws IllegalArgumentException if the url is not valid
 */
fun parseAddress(url: String): ParsedAddress {
    val regex = Regex("^(.+)@(\\d+).(\\d+).(\\d+).(\\d+):(\\d+)$")

    val match = regex.matchEntire(url)

    if (match == null) {
        throw IllegalArgumentException("Url does not match \"room@ipv4:port\" format")
    }

    val room = match.groups[1]!!.value
    val ipBytes = (2 until 6).map { match.groups[it]!!.value.toIntOrNull() }
    val port = match.groups[6]!!.value

    // iterate from 0 to len of ipBytes
    for (i in 0 until ipBytes.size) {
        val byte = ipBytes[i]
        if (byte == null || byte < 0 || byte > 255) {
            throw IllegalArgumentException("${cardinals[i]} IP byte outside range [0, 255]")
        }
    }

    val portInt = port.toIntOrNull()
    if (portInt == null || portInt < 0 || portInt > 65535) {
        throw IllegalArgumentException("Port must be between 0 and 65535")
    }

    return ParsedAddress(room, ipBytes.joinToString("."), port)
}

data class ParsedAddress(
    val room: String,
    val ip: String,
    val port: String,
);