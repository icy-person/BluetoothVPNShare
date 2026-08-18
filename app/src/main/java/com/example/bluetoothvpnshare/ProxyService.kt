package com.example.bluetoothvpnshare

import android.app.*
import android.content.Intent
import android.os.IBinder

class ProxyService : Service() {
    companion object {
        private const val CHANNEL = "bluetooth_vpn_share"
        private const val ID = 42
        init { System.loadLibrary("bluetooth_vpn_share") }
        @JvmStatic external fun nativeStart(port: Int, user: String, pass: String): Boolean
        @JvmStatic external fun nativeStop()
    }

    override fun onCreate() {
        super.onCreate()
        getSystemService(NotificationManager::class.java).createNotificationChannel(
            NotificationChannel(CHANNEL, "Bluetooth VPN Share", NotificationManager.IMPORTANCE_LOW)
        )
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val port = intent?.getIntExtra("port", 1080) ?: 1080
        val user = intent?.getStringExtra("user") ?: ""
        val pass = intent?.getStringExtra("pass") ?: ""
        startForeground(ID, Notification.Builder(this, CHANNEL)
            .setContentTitle("Bluetooth VPN Share")
            .setContentText("Proxy listening on :$port")
            .setSmallIcon(android.R.drawable.stat_sys_upload_done)
            .setOngoing(true)
            .build())
        nativeStart(port, user, pass)
        return START_STICKY
    }

    override fun onDestroy() {
        nativeStop()
        super.onDestroy()
    }
    override fun onBind(intent: Intent?): IBinder? = null
}
