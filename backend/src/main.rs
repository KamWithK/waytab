#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::os::raw::c_int;
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const HOST: &str = "localhost";
const PORT: &str = "9002";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PointerEventType {
    #[serde(rename = "pointerdown")]
    Down,
    #[serde(rename = "pointerup")]
    Up,
    #[serde(rename = "pointercancel")]
    Cancel,
    #[serde(rename = "pointermove")]
    Move,
    #[serde(rename = "pointerover")]
    Over,
    #[serde(rename = "pointerenter")]
    Enter,
    #[serde(rename = "pointerleave")]
    Leave,
    #[serde(rename = "pointerout")]
    Out,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PointerType {
    #[serde(rename = "")]
    NoDetect,
    #[serde(rename = "mouse")]
    Mouse,
    #[serde(rename = "pen")]
    Pen,
    #[serde(rename = "touch")]
    Touch,
}

#[derive(Serialize, Deserialize, Debug)]
struct PointerEventMessage {
    event_type: PointerEventType,
    pointer_id: i64,
    timestamp: u64,
    pointer_type: PointerType,
    buttons: Number,
    x: f64,
    y: f64,
    pressure: f64,
    tilt_x: i32,
    tilt_y: i32,
    width: f64,
    height: f64,
}

fn send_event(fd: c_int, type_: c_int, code: c_int, value: c_int) {
    unsafe { emit_event(fd, type_, code, value) };
}

async fn make_connections(fd: c_int) {
    if let Ok(server) = TcpListener::bind(format!("{}:{}", HOST, PORT)).await {
        while let Ok((stream, _)) = server.accept().await {
            tokio::spawn(accept_connection(stream, fd));
        }
    }
}

fn handlePointerType(event: PointerEventMessage, fd: c_int) {
    match event.pointer_type {
        PointerType::Pen => handlePen(event, fd),
        PointerType::NoDetect => notYetHandled(event, fd),
        PointerType::Mouse => notYetHandled(event, fd),
        PointerType::Touch => notYetHandled(event, fd),
    }
}

fn handlePen(event: PointerEventMessage, fd: c_int) {
    match event.event_type {
        PointerEventType::Down
        | PointerEventType::Move
        | PointerEventType::Over
        | PointerEventType::Enter => handleMove(event, fd),
        PointerEventType::Up
        | PointerEventType::Cancel
        | PointerEventType::Leave
        | PointerEventType::Out => handleEnd(event, fd),
    }
}

fn notYetHandled(event: PointerEventMessage, _fd: c_int) {
    println!(
        "Pointer: {:#?}, Event: {:#?}",
        event.pointer_type, event.event_type
    );
}

fn handleMove(event: PointerEventMessage, fd: c_int) {
    let timestamp = (event.timestamp % (i32::MAX as u64 + 1)) as i32;
    let x = (event.x * (ABS_MAXVAL as f64)) as i32;
    let y = (event.y * (ABS_MAXVAL as f64)) as i32;
    let pressure = (event.pressure * (ABS_MAXVAL as f64)) as i32;

    if PointerEventType::Down == event.event_type {
        send_event(fd, EV_KEY, BTN_TOOL_PEN, 1);
    }

    send_event(fd, EV_ABS, ABS_X, x);
    send_event(fd, EV_ABS, ABS_Y, y);
    send_event(fd, EV_ABS, ABS_PRESSURE, pressure);
    send_event(fd, EV_ABS, ABS_TILT_X, event.tilt_x);
    send_event(fd, EV_ABS, ABS_TILT_Y, event.tilt_y);
    send_event(fd, EV_MSC, MSC_TIMESTAMP, timestamp);
    send_event(fd, EV_SYN, SYN_REPORT, 1);
}

fn handleEnd(event: PointerEventMessage, fd: c_int) {}

async fn accept_connection(stream: TcpStream, fd: c_int) {
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        let (_, read) = ws_stream.split();
        read.for_each(|msg| async {
            let Ok(Message::Text(msg)) = msg else {
                return;
            };

            let event = serde_json::from_str::<PointerEventMessage>(msg.as_str()).unwrap();
            handlePointerType(event, fd);

            // send_event(fd, EV_KEY, BTN_TOOL_PEN, 1);
            // send_event(fd, EV_KEY, BTN_TOUCH, 1);
        })
        .await;
    }
}

#[tokio::main]
async fn main() {
    let fd = unsafe { create_stylus() };

    make_connections(fd).await;
}
