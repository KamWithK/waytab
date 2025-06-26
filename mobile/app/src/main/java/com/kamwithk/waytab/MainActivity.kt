package com.kamwithk.waytab

import android.content.res.Resources
import android.os.Bundle
import android.util.Log
import android.view.MotionEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
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
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.serialization.SerialName
import kotlinx.serialization.json.Json
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

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        WindowCompat.getInsetsController(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            hide(WindowInsetsCompat.Type.systemBars())
        }

        setContent {
            Box(Modifier.fillMaxSize().background(Color.Magenta)) {
                AndroidView(
                    factory = { context ->
                        RtspSurfaceView(context).apply {
                            val uri = "rtsp://rtsp.kamwithk.com:8554".toUri()
                            init(uri, null, null, null)
                            start(
                                requestVideo = true,
                                requestAudio = false,
                                requestApplication = false,
                            )

                            //                            stop()
                        }
                    }
                )

                Box(
                    modifier =
                        Modifier.fillMaxSize().background(Color.Transparent).pointerInteropFilter {
                            event ->
                            handleMotion(event)
                        }
                )
            }
        }

        lifecycleScope.launch(Dispatchers.IO) {
            try {
                session = client.webSocketSession(host = "websocket.kamwithk.com", path = "/wss")
            } catch (e: Exception) {
                Log.e("WebSocket", "Failed to connect:", e)
            }
        }

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
