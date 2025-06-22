use futures_util::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};
use tungstenite::Message;

use crate::uinput::PointerEventMessage;

const HOST: &str = "localhost";
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
) {
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        let (_, read) = ws_stream.split();
        read.for_each(|msg| async {
            let Ok(Message::Text(msg)) = msg else {
                return;
            };

            stylus_sender.send(serde_json::from_str::<PointerEventMessage>(msg.as_str()).unwrap());
        })
        .await;
    }
}
