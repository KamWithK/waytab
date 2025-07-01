use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::Result;
use tokio::net::UdpSocket;

const SERVER_SEND_MESSAGE: &[u8; 6] = b"waytab";

pub(crate) async fn connect_clients() -> Result<()> {
    let multicast_addr = Ipv4Addr::new(239, 115, 3, 2);
    let multicast_port = 4819;
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, multicast_port);
    let socket = UdpSocket::bind(socket_addr).await?;
    socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = [0u8; 1];

    loop {
        let (_len, client_addr) = socket.recv_from(&mut buf).await?;

        socket.send_to(SERVER_SEND_MESSAGE, &client_addr).await?;
    }
}
