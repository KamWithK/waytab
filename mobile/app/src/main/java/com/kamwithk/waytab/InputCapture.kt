package com.kamwithk.waytab

import android.view.MotionEvent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.layout.onGloballyPositioned
import kotlinx.serialization.SerialName
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

@Composable
fun InputCapture() {
    var width by remember { mutableIntStateOf(0) }
    var height by remember { mutableIntStateOf(0) }

    Box(
        modifier =
            Modifier.fillMaxSize()
                .onGloballyPositioned { coordinates ->
                    width = coordinates.size.width
                    height = coordinates.size.height
                }
                .background(Color.Transparent)
                .pointerInteropFilter { event ->
                    var message = handleMotion(event, width, height)
                    ServerController.queueMessage(message)
                    true
                }
    )
}

fun handleMotion(event: MotionEvent, width: Int, height: Int): MotionEventMessage {
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

    return MotionEventMessage(
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
}
