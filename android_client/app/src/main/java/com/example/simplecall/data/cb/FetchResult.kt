package com.example.simplecall.data.cb

interface FetchResult {
    fun onDataFetchedSuccess(urls: List<String>)
    fun onDataFetchedFailed(message: String)
}