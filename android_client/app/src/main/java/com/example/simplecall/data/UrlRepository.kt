package com.example.simplecall.data

class UrlRepository(private val urlDao: UrlDAO) {

    val allUrls = urlDao.getAllUrls()

    fun insert(url: String) {
        AppDatabase.databaseWriteExecutor.execute {
            urlDao.deleteByUrl(url)
            urlDao.insert(UrlModel(url = url))
        }
    }
}