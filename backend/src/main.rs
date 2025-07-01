mod comms;
mod networking;
mod stream;
mod uinput;

use anyhow::Result;
use comms::make_connections;
use networking::connect_clients;
use stream::initiate_stream;
use tokio::sync::broadcast;
use uinput::{PointerEventMessage, handle_inputs};

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(connect_clients());

    let (stylus_sender, stylus_receiver) = broadcast::channel::<PointerEventMessage>(64);

    let websocket_handle = tokio::spawn(make_connections(stylus_sender));
    let stylus_handle = tokio::spawn(handle_inputs(stylus_receiver));
    let stream_handle = tokio::spawn(initiate_stream());

    tokio::try_join!(websocket_handle, stylus_handle, stream_handle)?.2?;

    Ok(())
}
