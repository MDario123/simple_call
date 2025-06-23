package com.example.simplecall.data

import androidx.lifecycle.LiveData
import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query

@Dao
interface UrlDAO {
    @Query("SELECT id, url FROM urls ORDER BY id Desc")
    fun getAllUrls(): LiveData<List<UrlModel>>

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(url: UrlModel)

    @Query("DELETE FROM urls WHERE url = :url")
    fun deleteByUrl(url: String)
}