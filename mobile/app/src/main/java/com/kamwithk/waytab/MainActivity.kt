package com.kamwithk.waytab

import android.net.wifi.WifiManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        ServerController.wifiManager = getSystemService(WIFI_SERVICE) as WifiManager

        WindowCompat.getInsetsController(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            hide(WindowInsetsCompat.Type.systemBars())
        }

        setContent {
            val serverAddress = ServerController.serverAddress.collectAsState()

            Box(Modifier.fillMaxSize().background(Color.Magenta)) {
                val address = serverAddress.value
                if (address != null) {
                    RtspScreen(address)
                }

                InputCapture()
            }
        }

        lifecycleScope.launch(Dispatchers.IO) { ServerController.handleLifecycle() }
        lifecycleScope.launch(Dispatchers.IO) { ServerController.processFlow() }
    }
}
