package com.kamwithk.waytab

import android.net.wifi.WifiManager
import android.util.Log
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.sendSerialized
import io.ktor.client.plugins.websocket.webSocketSession
import io.ktor.serialization.kotlinx.KotlinxWebsocketSerializationConverter
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.serialization.json.Json
import java.net.DatagramPacket
import java.net.InetAddress
import java.net.MulticastSocket
import java.net.SocketTimeoutException

object ServerConfig {
    const val MULTICAST_GROUP = "239.115.3.2"
    const val MULTICAST_PORT = 4819
    const val MULTICAST_MESSAGE = "request"
    const val SOCKET_TIMEOUT = 1000
    const val WEBSOCKET_PORT = 9002
    const val RTSP_PORT = 8554
}

object ServerController {
    lateinit var wifiManager: WifiManager

    val serverAddress = MutableStateFlow<InetAddress?>(null)
    private var client: HttpClient =
        HttpClient(CIO) {
            install(WebSockets) { contentConverter = KotlinxWebsocketSerializationConverter(Json) }
        }
    private var session: DefaultClientWebSocketSession? = null
    private val eventFlow =
        MutableSharedFlow<MotionEventMessage>(
            extraBufferCapacity = 64,
            onBufferOverflow = BufferOverflow.DROP_OLDEST,
        )

    suspend fun handleLifecycle() {
        while (true) {
            Log.d("server", "Connection Lifecycle Started")
            requestConnectServer()
            connectToWebSocket()

            val closeReason = session?.closeReason?.await()
            Log.e("websocket", "Disconnected with ${closeReason?.knownReason}")

            resetConnections()
            Log.d("server", "Connections reset")
        }
    }

    private fun requestConnectServer() {
        Log.d("multicast", "Requesting connection to server")

        val multicastLock = wifiManager.createMulticastLock("multicastLock")
        multicastLock.setReferenceCounted(true)
        multicastLock.acquire()

        var serverFound = false
        while (!serverFound) {
            serverFound = tryConnectServer()
        }

        multicastLock?.release()
    }

    private fun tryConnectServer(): Boolean {
        val group = InetAddress.getByName(ServerConfig.MULTICAST_GROUP)
        val socket = MulticastSocket(ServerConfig.MULTICAST_PORT)
        socket.soTimeout = ServerConfig.SOCKET_TIMEOUT

        val buf = ServerConfig.MULTICAST_MESSAGE.toByteArray()
        val packet = DatagramPacket(buf, buf.size, group, ServerConfig.MULTICAST_PORT)
        socket.send(packet)

        socket.joinGroup(group)

        var serverFound = true
        try {
            socket.receive(packet)
            serverAddress.value = packet.address
            Log.d("multicast", "Server at ${serverAddress.value?.hostAddress} responded")
        } catch (_: SocketTimeoutException) {
            Log.d("multicast", "Server not found")
            serverFound = false
        }

        socket.leaveGroup(group)
        socket.close()

        return serverFound
    }

    private suspend fun connectToWebSocket() {
        while (serverAddress.value == null) {
            delay(100)
        }
        val address = serverAddress.value ?: return

        try {
            Log.d(
                "websocket",
                "Trying to connect to ${address.hostAddress}:${ServerConfig.WEBSOCKET_PORT}",
            )
            session =
                client.webSocketSession(
                    host = address.hostAddress,
                    port = ServerConfig.WEBSOCKET_PORT,
                )
            Log.d("websocket", "Connected")
        } catch (e: Exception) {
            Log.e("websocket", "Failed to connect:", e)
        }
    }

    private fun resetConnections() {
        serverAddress.value = null
        session = null
    }

    suspend fun processFlow() {
        eventFlow.collectLatest { eventMessage ->
            try {
                session?.sendSerialized(eventMessage)
            } catch (e: Exception) {
                Log.e("WebSocket", "Failed to send event:", e)
            }
        }
    }

    fun queueMessage(message: MotionEventMessage) {
        eventFlow.tryEmit(message)
    }
}
