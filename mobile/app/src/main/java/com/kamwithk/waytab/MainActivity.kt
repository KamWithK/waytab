package com.kamwithk.waytab

import android.content.res.Resources
import android.net.wifi.WifiManager
import android.os.Bundle
import android.util.Log
import android.view.MotionEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import com.alexvas.rtsp.widget.RtspStatusListener
import com.alexvas.rtsp.widget.RtspSurfaceView
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.sendSerialized
import io.ktor.client.plugins.websocket.webSocketSession
import io.ktor.serialization.kotlinx.KotlinxWebsocketSerializationConverter
import io.ktor.websocket.close
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.serialization.SerialName
import kotlinx.serialization.json.Json
import java.net.DatagramPacket
import java.net.InetAddress
import java.net.MulticastSocket
import java.net.SocketTimeoutException
import kotlin.math.cos
import kotlin.math.sin

@kotlinx.serialization.Serializable
data class MotionEventMessage(
    @SerialName("event_type") val eventType: String,
    @SerialName("pointer_id") val pointerId: Int,
    @SerialName("timestamp") val timestamp: Int,
    @SerialName("pointer_type") val pointerType: String,
    @SerialName("buttons") val buttons: Int,
    @SerialName("x") val x: Float,
    @SerialName("y") val y: Float,
    @SerialName("pressure") val pressure: Float,
    @SerialName("tilt_x") val tiltX: Int,
    @SerialName("tilt_y") val tiltY: Int,
    @SerialName("touch_major") val touchMajor: Float,
    @SerialName("touch_minor") val touchMinor: Float,
)

class MainActivity : ComponentActivity() {
    private var client =
        HttpClient(CIO) {
            install(WebSockets) { contentConverter = KotlinxWebsocketSerializationConverter(Json) }
        }
    private var session: DefaultClientWebSocketSession? = null
    private val eventFlow =
        MutableSharedFlow<MotionEventMessage>(
            extraBufferCapacity = 64,
            onBufferOverflow = BufferOverflow.DROP_OLDEST,
        )

    private var width = Resources.getSystem().displayMetrics.widthPixels
    private var height = Resources.getSystem().displayMetrics.heightPixels

    private var serverAddress = mutableStateOf<InetAddress?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        WindowCompat.getInsetsController(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            hide(WindowInsetsCompat.Type.systemBars())
        }

        setContent {
            val address = serverAddress.value

            Box(Modifier.fillMaxSize().background(Color.Magenta)) {
                if (address != null) {
                    RtspScreenContent(address)
                }

                Box(
                    modifier =
                        Modifier.fillMaxSize().background(Color.Transparent).pointerInteropFilter {
                            event ->
                            handleMotion(event)
                        }
                )
            }
        }

        lifecycleScope.launch(Dispatchers.IO) { requestConnectServer() }

        lifecycleScope.launch(Dispatchers.IO) { connectToWebsocket() }

        lifecycleScope.launch {
            eventFlow.collectLatest { eventMessage ->
                try {
                    session?.sendSerialized(eventMessage)
                } catch (e: Exception) {
                    Log.e("WebSocket", "Failed to send event:", e)
                }
            }
        }
    }

    private suspend fun connectToWebsocket() {
        while (serverAddress.value == null) {
            delay(100)
        }
        val address = serverAddress.value

        try {
            Log.d("websocket", "Trying to connect to ${address?.hostAddress}:9002")
            session = client.webSocketSession(host = address?.hostAddress, port = 9002)
            Log.d("websocket", "Connected")
            session?.closeReason?.await()
            Log.d("websocket", "Disconnected")

            serverAddress.value = null
            Log.d("websocket", "Server connection reset")
            requestConnectServer()
            connectToWebsocket()
        } catch (e: Exception) {
            Log.e("websocket", "Failed to connect:", e)
        }
    }

    @Composable
    private fun RtspScreenContent(serverAddress: InetAddress) {
        AndroidView(
            factory = { context ->
                RtspSurfaceView(context).apply {
                    val uri = "rtsp://${serverAddress.hostAddress}:8554".toUri()
                    init(uri, null, null, null)
                    start(requestVideo = true, requestAudio = false, requestApplication = false)

                    setStatusListener(
                        object : RtspStatusListener {
                            override fun onRtspStatusFailed(message: String?) {
                                Log.d("rtsp", "Failed to connect to $uri")
                                start(
                                    requestVideo = true,
                                    requestAudio = false,
                                    requestApplication = false,
                                )
                            }

                            override fun onRtspStatusDisconnected() {
                                Log.d("rtsp", "Disconnected, will try to reconnect to $uri")
                                start(
                                    requestVideo = true,
                                    requestAudio = false,
                                    requestApplication = false,
                                )
                            }
                        }
                    )
                }
            }
        )
    }

    private fun requestConnectServer() {
        Log.d("multicast", "Requesting connection to server")

        val wifi = getSystemService(WIFI_SERVICE) as WifiManager
        val multicastLock = wifi.createMulticastLock("multicastLock")
        multicastLock.setReferenceCounted(true)
        multicastLock.acquire()

        val group = InetAddress.getByName("239.115.3.2")
        val socket = MulticastSocket(4819)
        socket.soTimeout = 1000

        val buf = "request".toByteArray()
        val packet = DatagramPacket(buf, buf.size, group, 4819)
        socket.send(packet)

        socket.joinGroup(group)

        var fail = false
        try {
            socket.receive(packet)
            serverAddress.value = packet.address
            Log.d("multicast", "Server at ${serverAddress.value?.hostAddress} responded")
        } catch (_: SocketTimeoutException) {
            Log.d("multicast", "Server not found")
            fail = true
        }

        socket.leaveGroup(group)
        socket.close()
        multicastLock?.release()

        Log.d("multicast", "Server retry triggered")
        if (fail) requestConnectServer()
    }

    override fun onDestroy() {
        super.onDestroy()
        lifecycleScope.launch {
            session?.close()
            client.close()
        }
    }

    fun handleMotion(event: MotionEvent?): Boolean {
        if (event == null || session == null) return false

        val toolType =
            when (event.getToolType(0)) {
                MotionEvent.TOOL_TYPE_FINGER -> "touch"
                MotionEvent.TOOL_TYPE_STYLUS -> "pen"
                MotionEvent.TOOL_TYPE_ERASER -> "eraser"
                MotionEvent.TOOL_TYPE_MOUSE -> "mouse"
                else -> ""
            }

        // TODO: Approximation but why do we flick between up and the angle?
        val tiltMagnitude = sin(event.getAxisValue(MotionEvent.AXIS_TILT))
        val tiltAngle = event.orientation
        val tiltX = sin(tiltAngle) * tiltMagnitude * -0.5
        val tiltY = cos(tiltAngle) * tiltMagnitude * 0.5

        val motionEventMessage =
            MotionEventMessage(
                MotionEvent.actionToString(event.action),
                event.getPointerId(event.actionIndex),
                event.eventTime.toInt(),
                toolType,
                event.buttonState,
                event.x / width.toFloat(),
                event.y / height.toFloat(),
                event.pressure,
                Math.toDegrees(tiltX.toDouble()).toInt(),
                Math.toDegrees(tiltY.toDouble()).toInt(),
                event.touchMajor,
                event.toolMinor,
            )

        eventFlow.tryEmit(motionEventMessage)

        return true
    }
}
