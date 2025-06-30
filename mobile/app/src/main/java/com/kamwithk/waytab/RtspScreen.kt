package com.kamwithk.waytab

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import com.alexvas.rtsp.widget.RtspStatusListener
import com.alexvas.rtsp.widget.RtspSurfaceView
import java.net.InetAddress

@Composable
fun RtspScreen(serverAddress: InetAddress) {
    key(serverAddress) {
        AndroidView(
            factory = { context ->
                RtspSurfaceView(context).apply {
                    val uri = "rtsp://${serverAddress.hostAddress}:${ServerConfig.RTSP_PORT}".toUri()
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
}
