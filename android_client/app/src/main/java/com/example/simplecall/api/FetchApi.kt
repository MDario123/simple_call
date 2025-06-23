package com.example.simplecall.api

import retrofit2.Call
import retrofit2.http.GET

interface FetchApi {

    @GET("urls")
    fun fetchUrls(): Call<List<String>>
}