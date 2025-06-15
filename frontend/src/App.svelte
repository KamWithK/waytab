<script lang="ts">
  const socket = new WebSocket("wss://websocket.kamwithk.com");

  interface PointerEventMessage {
    event_type: string;
    pointer_id: number;
    timestamp: number;
    pointer_type: string;
    buttons: number;
    x: number;
    y: number;
    pressure: number;
    tilt_x: number;
    tilt_y: number;
    width: number;
    height: number;
  }

  const sendEvent = (event: PointerEvent) => {
    if (!socket.readyState) return;

    const target = event.target as HTMLElement;
    const boundingBox = target.getBoundingClientRect();
    const diagonal = Math.sqrt(
      Math.pow(boundingBox.width, 2) + Math.pow(boundingBox.height, 2),
    );

    const absoluteX = event.clientX / boundingBox.width;
    const absoluteY = event.clientY / boundingBox.height;
    const absoluteWidth = event.width / diagonal;
    const absoluteHeight = event.height / diagonal;

    const pointerEventMessage: PointerEventMessage = {
      event_type: event.type,
      pointer_id: event.pointerId,
      timestamp: Math.round(event.timeStamp * 1000),
      pointer_type: event.pointerType,
      buttons: event.buttons,
      x: absoluteX,
      y: absoluteY,
      pressure: event.pressure,
      tilt_x: Math.round(event.tiltX),
      tilt_y: Math.round(event.tiltY),
      width: absoluteWidth,
      height: absoluteHeight,
    };

    socket.send(JSON.stringify(pointerEventMessage));
  };
</script>

<canvas
  onpointerdown={sendEvent}
  onpointerup={sendEvent}
  onpointercancel={sendEvent}
  onpointermove={sendEvent}
  onpointerout={sendEvent}
  onpointerleave={sendEvent}
  onpointerenter={sendEvent}
  onpointerover={sendEvent}
></canvas>

<style>
  canvas {
    width: 100%;
    height: 100%;
    background-color: purple;
  }
</style>
