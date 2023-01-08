use std::io::Error;
use std::io::ErrorKind;
use std::env;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{io, io::Write};
use tokio_util::codec::Framed;
use tun::TunPacketCodec;
use packet::ip::Packet;
use futures::StreamExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:22168".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("listener accepted");
        let codec = TunPacketCodec::new(false, 1500);
        let mut stream = Framed::new(socket, codec);
        tokio::spawn(async move {
            while let Some(packet) = stream.next().await {
                match packet {
                    Ok(pkt) => println!("pkt: {:#?}", Packet::unchecked(pkt.get_bytes())),
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
        });
    }
    Ok(())
}