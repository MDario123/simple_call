package com.mdario.simplecall.api

import com.mdario.simplecall.data.cb.FetchResult
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import retrofit2.Call
import retrofit2.Callback
import retrofit2.Response
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory
import retrofit2.create

private val BASE_URL = "http://194.163.172.93:8280"

private val retrofit by lazy {
    val moshi = Moshi.Builder()
        .add(KotlinJsonAdapterFactory())
        .build()

    Retrofit.Builder()
        .baseUrl(BASE_URL)
        .addConverterFactory(MoshiConverterFactory.create(moshi))
        .build()
        .create<FetchApi>()
}

class FetchProvider {

    fun fetchUrls(cb: FetchResult) {
        retrofit.fetchUrls().enqueue(object : Callback<List<String>> {
            override fun onResponse(
                call: Call<List<String>>,
                response: Response<List<String>>
            ) {
                val output = response.body()
                if (response.isSuccessful && output != null) {
                    cb.onDataFetchedSuccess(output)
                } else {
                    cb.onDataFetchedFailed("${response.code()}: ${response.errorBody()}")
                }
            }

            override fun onFailure(call: Call<List<String>>, t: Throwable) {
                cb.onDataFetchedFailed("Error: ${t.message}")
            }
        })
    }
}