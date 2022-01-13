//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::io;
use std::io::{ErrorKind, IoSlice, Read, Write, Error};

use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;
use futures_core::ready;

use crate::r#async::win::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::Framed;
use wintun::Session;

use crate::device::Device as D;
use crate::platform::{Device, Queue};
use crate::platform::windows::TryWrite;
use crate::r#async::codec::*;

/// An async TUN device wrapper around a TUN device.
pub struct AsyncDevice {
    inner: AsyncFd<Device>,
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> io::Result<AsyncDevice> {
        Ok(AsyncDevice {
            inner: AsyncFd::new(device)?,
        })
    }
    /// Returns a shared reference to the underlying Device object
    pub fn get_ref(&self) -> &Device {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying Device object
    pub fn get_mut(&mut self) -> &mut Device {
        self.inner.get_mut()
    }

    /// Consumes this AsyncDevice and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let codec = TunPacketCodec::new(false, self.get_ref().mtu().unwrap_or(1504));
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let rbuf = buf.initialize_unfilled();
        match self.inner.try_read(rbuf) {
            Ok(0) => {
                println!("pending");
                return Poll::Ready(Ok(()))
            },
            Ok(n) => {
                println!("try_read");
                buf.advance(n);
                return Poll::Ready(Ok(()))
            },
            Err(e) => {
                println!("eerr");
                return Poll::Ready(Err(e))
            },
        }
    }
}

impl AsyncWrite for AsyncDevice {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.inner.try_write(buf) {
            Ok(n) => return Poll::Ready(Ok(n)),
            Err(e) => return Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// An async TUN device queue wrapper around a TUN device queue.
pub struct AsyncQueue {
    inner: AsyncFd<Queue>,
}

impl AsyncQueue {
    /// Create a new `AsyncQueue` wrapping around a `Queue`.
    pub fn new(queue: Queue) -> io::Result<AsyncQueue> {
        Ok(AsyncQueue {
            inner: AsyncFd::new(queue)?,
        })
    }
    /// Returns a shared reference to the underlying Queue object
    pub fn get_ref(&self) -> &Queue {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying Queue object
    pub fn get_mut(&mut self) -> &mut Queue {
        self.inner.get_mut()
    }

    /// Consumes this AsyncQueue and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let codec = TunPacketCodec::new(false, 1504);
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncQueue {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let rbuf = buf.initialize_unfilled();
        match self.inner.try_read(rbuf) {
            Ok(0) => return Poll::Ready(Ok(())),
            Ok(n) => {
                buf.advance(n);
                return Poll::Ready(Ok(()));
            }
            Err(e) => return Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for AsyncQueue {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.inner.try_write(buf) {
            Ok(n) => return Poll::Ready(Ok(n)),
            Err(e) => return Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub struct AsyncDevice2 {
    inner: Device,
}


impl AsyncDevice2 {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> io::Result<AsyncDevice2> {
        Ok(AsyncDevice2 {
            inner: device,
        })
    }
    /// Returns a shared reference to the underlying Device object
    pub fn get_ref(&self) -> &Device {
        &self.inner
    }

    /// Returns a mutable reference to the underlying Device object
    pub fn get_mut(&mut self) -> &mut Device {
        &mut self.inner
    }

    /// Consumes this AsyncDevice and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let codec = TunPacketCodec::new(false, self.get_ref().mtu().unwrap_or(1504));
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncDevice2 {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let rbuf = buf.initialize_unfilled();
        match Pin::new(&mut self.inner).poll_read(cx, rbuf) {
            Poll::Ready(Ok(n)) => {
                buf.advance(n);
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for AsyncDevice2 {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        match self.inner.try_write(buf) {
            Ok(n) => return Poll::Ready(Ok(n)),
            Err(e) => return Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct AsyncQueue2 {
    inner: Queue,
}

impl AsyncQueue2 {
    /// Create a new `AsyncQueue` wrapping around a `Queue`.
    pub fn new(queue: Queue) -> io::Result<AsyncQueue2> {
        Ok(AsyncQueue2 {
            inner: queue,
        })
    }
    /// Returns a shared reference to the underlying Queue object
    pub fn get_ref(&self) -> &Queue {
        &self.inner
    }

    /// Returns a mutable reference to the underlying Queue object
    pub fn get_mut(&mut self) -> &mut Queue {
        &mut self.inner
    }

    /// Consumes this AsyncQueue and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let codec = TunPacketCodec::new(false, 1504);
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncQueue2 {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let rbuf = buf.initialize_unfilled();
        match Pin::new(&mut self.inner).poll_read(cx, rbuf) {
            Poll::Ready(Ok(n)) =>{
                buf.advance(n);
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for AsyncQueue2 {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        match self.inner.try_write(buf) {
            Ok(n) => return Poll::Ready(Ok(n)),
            Err(e) => return Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}