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

use std::io::{self, Read, Write};
use std::thread;
use std::net::Ipv4Addr;
use std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle};
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use crate::configuration::Configuration;
use crate::device::Device as D;
use crate::error::*;
use wintun::Session;
use std::process::Command;
use std::task::{Context, Poll};
use std::pin::Pin;

/// A TUN device using the wintun driver.
pub struct Device {
    queue: Queue,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let wintun = unsafe { wintun::load() }.expect("Failed to load wintun dll");
        let n = config.name.clone().unwrap_or("wintun".to_string());
        let name = n.clone();
        let adapter = match wintun::Adapter::open(&wintun, name.as_str()) {
            Ok(a) => a,
            Err(_) => wintun::Adapter::create(&wintun, name.as_str(), name.as_str(), None)
                .expect("Failed to create wintun adapter!"),
        };
        let session = Arc::new(
            adapter
                .start_session(wintun::MAX_RING_CAPACITY)
                .expect("Failed to create session")
        );

        let address = config.address.clone()
            .map_or_else(|| "10.1.0.2".to_string(), |a| a.to_string());
        let destination = config.destination.clone()
            .map_or_else(|| "".to_string(), |a| a.to_string());
        let netmask = config.netmask.clone()
            .map_or_else(|| "255.255.255.0".to_string(), |a| a.to_string());
        let mtu = config.mtu.unwrap_or(1500);

        let queue = Queue {
            session,
            cached: Arc::new(Mutex::new(Vec::with_capacity(mtu as usize))),
        };
        let mut device = Self { queue };

        device.configure(&config)?;
        let out = Command::new("netsh")
            .arg("interface").arg("ipv4").arg("set").arg("address")
            .arg(name.as_str())
            .arg("static")
            .arg(address)
            .arg(netmask)
            .arg(destination)
            .arg("store=active")
            .output()
            .expect("failed to execute command");
        assert!(out.status.success());

        Ok(device)
    }

    pub fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.queue).poll_read(cx, buf)
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return self.queue.read(buf);
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.queue.read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.queue.write(buf);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.queue.flush();
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.queue.write_vectored(bufs)
    }
}

impl D for Device {
    type Queue = Queue;

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn broadcast(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn mtu(&self) -> Result<i32> {
        Ok(1500)
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        Ok(())
    }

    fn queue(&mut self, index: usize) -> Option<&mut Self::Queue> {
        return Some(&mut self.queue);
    }
}

pub struct Queue {
    session: Arc<Session>,
    cached: Arc<Mutex<Vec<u8>>>,
}

impl Queue {
    pub fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        {
            let mut guard = self.cached.lock().unwrap();
            if guard.len() > 0 {
                let res = match io::copy(&mut guard.as_slice(), &mut buf) {
                    Ok(n) => Poll::Ready(Ok(n as usize)),
                    Err(e) => Poll::Ready(Err(e))
                };
                guard.clear();
                return res;
            }
        }
        let reader_session = self.session.clone();
        match reader_session.try_receive() {
            Err(_) => Poll::Ready(Err(io::Error::from(io::ErrorKind::Other))),
            Ok(Some(packet)) => {
                match io::copy(&mut packet.bytes(), &mut buf) {
                    Ok(n) => Poll::Ready(Ok(n as usize)),
                    Err(e) => Poll::Ready(Err(e))
                }
            }
            Ok(None) => {
                let waker = cx.waker().clone();
                let cached = self.cached.clone();
                thread::spawn(move || {
                    let mut guard = cached.lock().unwrap();
                    match reader_session.receive_blocking() {
                        Ok(packet) => guard.extend_from_slice(&packet.bytes()),
                        Err(e) => {}
                    }
                    waker.wake()
                });
                Poll::Pending
            }
        }
    }

    fn try_read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let reader_session = self.session.clone();
        match reader_session.try_receive() {
            Err(_) => Err(io::Error::from(io::ErrorKind::Other)),
            Ok(op) => match op {
                None => Ok(0),
                Some(packet) => {
                    match io::copy(&mut packet.bytes(), &mut buf) {
                        Ok(s) => Ok(s as usize),
                        Err(e) => Err(e)
                    }
                }
            }
        }
    }
}

impl AsRawHandle for Queue {
    fn as_raw_handle(&self) -> RawHandle {
        todo!()
    }
}

impl IntoRawHandle for Queue {
    fn into_raw_handle(self) -> RawHandle {
        todo!()
    }
}

impl Read for Queue {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let reader_session = self.session.clone();
        match reader_session.receive_blocking() {
            Ok(pkt) => {
                match io::copy(&mut pkt.bytes(), &mut buf) {
                    Ok(n) => Ok(n as usize),
                    Err(e) => Err(e)
                }
            }
            Err(_) => Err(io::Error::from(io::ErrorKind::ConnectionAborted))
        }
    }
}

impl Write for Queue {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        let writer_session = self.session.clone();
        match writer_session.allocate_send_packet(size as u16) {
            Err(_) => Err(io::Error::from(io::ErrorKind::OutOfMemory)),
            Ok(mut packet) => {
                match io::copy(&mut buf, &mut packet.bytes_mut()) {
                    Ok(s) => {
                        writer_session.send_packet(packet);
                        Ok(s as usize)
                    }
                    Err(e) => Err(e)
                }
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.session.shutdown();
    }
}
