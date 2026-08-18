package com.example.bluetoothvpnshare

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.text.InputType
import android.view.ViewGroup
import android.widget.*

class MainActivity : Activity() {
    private lateinit var status: TextView
    private lateinit var port: EditText
    private lateinit var user: EditText
    private lateinit var pass: EditText

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(28, 28, 28, 28)
        }

        fun lp() = LinearLayout.LayoutParams(-1, -2)

        root.addView(TextView(this).apply {
            text = "Bluetooth VPN Share"
            textSize = 28f
        }, lp())
        root.addView(TextView(this).apply {
            text = "Bluetooth PAN → Rust proxy → existing phone VPN"
            textSize = 16f
            setPadding(0, 12, 0, 18)
        }, lp())

        port = EditText(this).apply {
            hint = "Port"
            setText("1080")
            inputType = InputType.TYPE_CLASS_NUMBER
        }
        root.addView(port, lp())

        user = EditText(this).apply { hint = "Username (optional)" }
        root.addView(user, lp())

        pass = EditText(this).apply {
            hint = "Password (optional)"
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        }
        root.addView(pass, lp())

        status = TextView(this).apply {
            text = "Stopped"
            textSize = 16f
            setPadding(0, 16, 0, 16)
        }
        root.addView(status, lp())

        val start = Button(this).apply {
            text = "Start"
            setOnClickListener {
                val p = port.text.toString().toIntOrNull()
                if (p == null || p !in 1024..65535) {
                    Toast.makeText(this@MainActivity, "Port must be 1024–65535", Toast.LENGTH_SHORT).show()
                    return@setOnClickListener
                }
                val i = Intent(this@MainActivity, ProxyService::class.java)
                    .putExtra("port", p)
                    .putExtra("user", user.text.toString())
                    .putExtra("pass", pass.text.toString())
                startForegroundService(i)
                status.text = "Running — SOCKS5 + HTTP :$p"
            }
        }
        root.addView(start, lp())

        root.addView(Button(this).apply {
            text = "Stop"
            setOnClickListener {
                stopService(Intent(this@MainActivity, ProxyService::class.java))
                status.text = "Stopped"
            }
        }, lp())

        root.addView(TextView(this).apply {
            text = "Important: configure the laptop to use this proxy. For all-device traffic, use a local tun2socks/transparent adapter on Linux."
            setPadding(0, 20, 0, 0)
        }, lp())

        setContentView(root)
    }
}
