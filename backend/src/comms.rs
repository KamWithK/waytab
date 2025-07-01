use anyhow::Result;
use futures_util::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};
use tungstenite::Message;

use crate::uinput::PointerEventMessage;

const HOST: &str = "0.0.0.0";
const PORT: &str = "9002";

pub(crate) async fn make_connections(stylus_sender: Sender<PointerEventMessage>) {
    if let Ok(server) = TcpListener::bind(format!("{}:{}", HOST, PORT)).await {
        while let Ok((stream, _)) = server.accept().await {
            tokio::spawn(accept_connection(stream, stylus_sender.clone()));
        }
    }
}

pub(crate) async fn accept_connection(
    stream: TcpStream,
    stylus_sender: Sender<PointerEventMessage>,
) -> Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        if let Message::Text(text) = msg? {
            let json = serde_json::from_str::<PointerEventMessage>(&text)?;
            stylus_sender.send(json)?;
        }
    }

    Ok(())
}
