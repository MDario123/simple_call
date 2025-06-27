package com.mdario.simplecall

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import com.mdario.simplecall.api.FetchProvider
import com.mdario.simplecall.data.MainViewModel
import com.mdario.simplecall.data.cb.FetchResult
import com.mdario.simplecall.ui.MainScreen
import com.mdario.simplecall.ui.theme.SimpleCallTheme

class MainActivity : ComponentActivity(), FetchResult {

    lateinit var recommendedRooms: MutableState<List<String>>
    lateinit var recentRooms: MutableState<List<String>>

    private val mainViewModel: MainViewModel by viewModels()

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        recommendedRooms = mutableStateOf(listOf())
        recentRooms = mutableStateOf(listOf())

        mainViewModel.getUrlsFromDatabase().observe(this) {
            urls -> recentRooms.value = urls.map {it.url}
        }


        var provider = FetchProvider()
        provider.fetchUrls(this)

        setContent {

            val roomsToShow = (recentRooms.value + recommendedRooms.value).distinct()

            SimpleCallTheme {
                Scaffold(
                    topBar = {
                        TopAppBar(
                            title = {
                                Text("SimpleCall")
                            }
                        )
                    }
                ) { padding ->
                    Column(modifier = Modifier.padding(padding)) {
                        MainScreen(this@MainActivity, mainViewModel, roomsToShow)
                    }
                }
            }
        }
    }

    override fun onDataFetchedSuccess(urls: List<String>) {
        recommendedRooms.value = urls
    }

    override fun onDataFetchedFailed(message: String) {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

}