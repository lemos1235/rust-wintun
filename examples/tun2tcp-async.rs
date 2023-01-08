use futures::StreamExt;
use packet::ip::Packet;
use tokio::net::{TcpSocket, UdpSocket};
use std::net::SocketAddr;
use std::sync::Arc;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::Error;
use std::future::Future;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut config = tun::Configuration::default();
    config
        .address((10, 0, 0, 2))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();
    #[cfg(target_os = "linux")]
        config.platform(|config| {
        config.packet_information(true);
    });
    let mut dev = tun::create_as_async(&config).unwrap();
    let sock = TcpSocket::new_v4()?;
    let mut stream = sock.connect("127.0.0.1:22168".parse().unwrap()).await?;
    println!("started");
    let (u, d) = tokio::io::copy_bidirectional(&mut dev, &mut stream).await?;
    println!("up: {}, down: {}", u, d);
    stream.shutdown().await?;
    Ok(())
}