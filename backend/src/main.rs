mod comms;
mod stream;
mod uinput;

use comms::make_connections;
use stream::initiate_stream;
use tokio::sync::broadcast;
use uinput::{PointerEventMessage, handle_inputs};

#[tokio::main]
async fn main() {
    let (stylus_sender, stylus_receiver) = broadcast::channel::<PointerEventMessage>(64);

    let websocket_handle = tokio::spawn(make_connections(stylus_sender));
    let stylus_handle = tokio::spawn(handle_inputs(stylus_receiver));
    let stream_handle = tokio::spawn(initiate_stream());

    tokio::try_join!(websocket_handle, stylus_handle, stream_handle).unwrap();
}
