use std::ffi::c_int;

use bindings::{
    ABS_MAXVAL, ABS_PRESSURE, ABS_TILT_X, ABS_TILT_Y, ABS_X, ABS_Y, BTN_TOOL_PEN, EV_ABS, EV_KEY,
    EV_MSC, EV_SYN, MSC_TIMESTAMP, SYN_REPORT, create_stylus, emit_event,
};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use tokio::sync::broadcast::Receiver;

mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PointerEventType {
    #[serde(rename = "ACTION_DOWN")]
    Down,
    #[serde(rename = "ACTION_UP")]
    Up,
    #[serde(rename = "ACTION_CANCEL")]
    Cancel,
    #[serde(rename = "ACTION_MOVE")]
    Move,
    #[serde(rename = "ACTION_HOVER_MOVE")]
    Over,
    #[serde(rename = "ACTION_HOVER_ENTER")]
    Enter,
    #[serde(rename = "ACTION_HOVER_EXIT")]
    Leave,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PointerEventMessage {
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
    touch_major: f64,
    touch_minor: f64,
}

pub(crate) async fn handle_inputs(mut stylus_receiver: Receiver<PointerEventMessage>) {
    let fd = unsafe { create_stylus() };

    while let Ok(pointer_event_message) = stylus_receiver.recv().await {
        handle_pointer_type(pointer_event_message, fd);
    }
}

fn send_event(fd: c_int, type_: c_int, code: c_int, value: c_int) {
    unsafe { emit_event(fd, type_, code, value) };
}

fn handle_pointer_type(event: PointerEventMessage, fd: c_int) {
    match event.pointer_type {
        PointerType::Pen => handle_pen(event, fd),
        PointerType::NoDetect => not_yet_handled(event, fd),
        PointerType::Mouse => not_yet_handled(event, fd),
        PointerType::Touch => not_yet_handled(event, fd),
    }
}

fn handle_pen(event: PointerEventMessage, fd: c_int) {
    match event.event_type {
        PointerEventType::Down
        | PointerEventType::Move
        | PointerEventType::Over
        | PointerEventType::Enter => handle_move(event, fd),
        PointerEventType::Up | PointerEventType::Cancel | PointerEventType::Leave => {
            handle_end(event, fd)
        }
    }
}

fn not_yet_handled(event: PointerEventMessage, _fd: c_int) {
    println!(
        "Pointer: {:#?}, Event: {:#?}",
        event.pointer_type, event.event_type
    );
}

fn handle_move(event: PointerEventMessage, fd: c_int) {
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

fn handle_end(_event: PointerEventMessage, _fd: c_int) {}
