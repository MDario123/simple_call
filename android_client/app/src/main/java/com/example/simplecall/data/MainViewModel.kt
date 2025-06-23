package com.example.simplecall.data

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.LiveData

class MainViewModel(application: Application) : AndroidViewModel(application) {

    val database = AppDatabase.getDatabase(application)

    val repository = UrlRepository(database.urlDao())

    fun getUrlsFromDatabase(): LiveData<List<UrlModel>> {
        return repository.allUrls
    }

    fun addUrl(url: String) {
        repository.insert(url)
    }

}