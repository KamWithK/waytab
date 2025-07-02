use bitflags::bitflags;
use std::ffi::c_int;

use bindings::{
    ABS_MAXVAL, ABS_PRESSURE, ABS_TILT_X, ABS_TILT_Y, ABS_X, ABS_Y, BTN_TOOL_PEN, EV_ABS, EV_KEY,
    EV_MSC, EV_SYN, MSC_TIMESTAMP, SYN_REPORT, create_stylus, emit_event,
};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::sync::broadcast::Receiver;

use crate::uinput::bindings::{BTN_STYLUS, BTN_STYLUS2, BTN_TOOL_RUBBER, BTN_TOUCH};

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PointerType {
    NoDetect = 0,
    Touch = 1,
    Stylus = 2,
    Mouse = 3,
    Eraser = 4,
}

bitflags! {
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ButtonActions: u8 {
        const PRIMARY = 0b000_0001;
        const SECONDARY = 0b000_0010;
        const TERTIARY = 0b000_0100;
        const FORWARD = 0b000_1000;
        const BACK = 0b001_0000;
        const STYLUS_PRIMARY = 0b010_0000;
        const STYLUS_SECONDARY = 0b100_0000;
    }
}
fn button_from<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ButtonActions, D::Error> {
    Ok(ButtonActions::from_bits_truncate(u8::deserialize(
        deserializer,
    )?))
}
fn pointer_from<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PointerType, D::Error> {
    let value = i8::deserialize(deserializer)?;
    match value {
        0 => Ok(PointerType::NoDetect),
        1 => Ok(PointerType::Touch),
        2 => Ok(PointerType::Stylus),
        3 => Ok(PointerType::Mouse),
        4 => Ok(PointerType::Eraser),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid PointerEventType: {}",
            value
        ))),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PointerEventMessage {
    event_type: PointerEventType,
    pointer_id: i64,
    timestamp: u64,
    #[serde(deserialize_with = "pointer_from")]
    pointer_type: PointerType,
    #[serde(deserialize_with = "button_from")]
    buttons: ButtonActions,
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
    let btn_code = match &event.pointer_type {
        PointerType::Stylus => Some(BTN_TOOL_PEN),
        PointerType::Eraser => Some(BTN_TOOL_RUBBER),
        PointerType::Touch => Some(BTN_TOUCH),
        _ => None,
    };
    if let Some(btn_code) = btn_code {
        send_event(fd, EV_KEY, btn_code, 1);
    }

    match &event.pointer_type {
        PointerType::Stylus | PointerType::Eraser => handle_pen(&event, fd),
        PointerType::Touch => not_yet_handled(&event, fd),
        PointerType::Mouse => (),
        PointerType::NoDetect => (),
    }
}

fn handle_pen(event: &PointerEventMessage, fd: c_int) {
    match event.event_type {
        PointerEventType::Down
        | PointerEventType::Move
        | PointerEventType::Over
        | PointerEventType::Enter => handle_move(event, fd),
        PointerEventType::Up | PointerEventType::Cancel | PointerEventType::Leave => {
            handle_end(event, fd)
        }
    }

    handle_buttons(event, fd);

    let timestamp = (event.timestamp % (i32::MAX as u64 + 1)) as i32;
    send_event(fd, EV_MSC, MSC_TIMESTAMP, timestamp);
    send_event(fd, EV_SYN, SYN_REPORT, 1);
}

fn not_yet_handled(event: &PointerEventMessage, _fd: c_int) {
    println!(
        "Pointer: {:#?}, Event: {:#?}",
        event.pointer_type, event.event_type
    );
}

fn handle_move(event: &PointerEventMessage, fd: c_int) {
    let x = (event.x * (ABS_MAXVAL as f64)) as i32;
    let y = (event.y * (ABS_MAXVAL as f64)) as i32;
    let pressure = (event.pressure * (ABS_MAXVAL as f64)) as i32;

    send_event(fd, EV_ABS, ABS_X, x);
    send_event(fd, EV_ABS, ABS_Y, y);
    send_event(fd, EV_ABS, ABS_PRESSURE, pressure);
    send_event(fd, EV_ABS, ABS_TILT_X, event.tilt_x);
    send_event(fd, EV_ABS, ABS_TILT_Y, event.tilt_y);
}

fn handle_buttons(event: &PointerEventMessage, fd: c_int) {
    let contains_stylus = event.buttons.contains(ButtonActions::STYLUS_PRIMARY);
    let contains_stylus2 = event.buttons.contains(ButtonActions::STYLUS_SECONDARY);

    send_event(fd, EV_KEY, BTN_STYLUS, contains_stylus.into());
    send_event(fd, EV_KEY, BTN_STYLUS2, contains_stylus2.into());
}

fn handle_end(_event: &PointerEventMessage, fd: c_int) {
    send_event(fd, EV_ABS, ABS_PRESSURE, 0);
}
