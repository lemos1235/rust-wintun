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

use std::ffi::{CStr, CString};
use std::io::{self, ErrorKind, Read, Write};
use std::mem;
use std::net::Ipv4Addr;
use std::ptr;
use std::sync::Arc;
use std::vec::Vec;

use crate::configuration::{Configuration, Layer};
use crate::device::Device as D;
use crate::error::*;
use wintun::{Session, Packet, WintunError, Wintun};
use packet;
use crate::platform::windows::{TryRead, TryWrite};
use ipconfig::{get_adapters, Adapter as IpAdapter};
use std::process::Command;
// use std::error::Error;

/// A TUN device using the wintun driver.
pub struct Device {
    name: String,
    queue: Queue,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let wintun = unsafe { wintun::load() }
            .expect("Failed to load wintun dll");
        let version = wintun::get_running_driver_version(&wintun);
        println!("Using wintun version: {:?}", version);

        let n = config.name.clone().unwrap_or("wintun".to_string());
        // let name = &n[..];
        let name = n.clone();
        let adapter = match wintun::Adapter::open(&wintun, name.as_str()) {
            Ok(a) => a,
            Err(_) => wintun::Adapter::create(&wintun, name.as_str(), name.as_str(), None)
                .expect("Failed to create wintun adapter!"),
        };
        let session = Arc::new(adapter.start_session(wintun::MAX_RING_CAPACITY).unwrap());

        let address =  config.address.clone().unwrap_or("10.0.0.2".parse().unwrap());
        let destination =  config.destination.clone().unwrap_or("10.0.0.1".parse().unwrap());
        let netmask =  config.netmask.clone().unwrap_or("255.255.255.0".parse().unwrap());

        let out = Command::new("netsh")
            .arg("interface").arg("ip").arg("set").arg("address")
            .arg(format!("name={}", n))
            .arg(format!("source={}", "static"))
            .arg(format!("address={}",address))
            .arg(format!("mask={}", netmask))
            .arg(format!("gateway={}", destination))
            .output()
            .expect("failed to execute command");
        println!("status: {}", out.status);
        io::stdout().write_all(&out.stdout).unwrap();
        io::stderr().write_all(&out.stderr).unwrap();
        assert!(out.status.success());
        // let out = String::from_utf8_lossy(&out.stdout).to_string();
        // let cols: Vec<&str> = out
        //     .lines()
        //     .filter(|l| l.contains("via"))
        //     .next()
        //     .unwrap()
        //     .split_whitespace()
        //     .map(str::trim)
        //     .collect();
        // assert!(cols.len() >= 3);
        // let res = cols[2].to_string();

        // let adapters = ipconfig::get_adapters().map_err(|e| Error::InvalidConfig )?;
        // println!("{}", adapters.len());
        // let iff= adapters.iter().filter(| &a| {
        //    return  a.friendly_name().eq( name.clone().as_str())
        // }).last().unwrap();
        // config.address
        // iff.ip_addresses()
        // println!("{:?}", iff.friendly_name());
        // adapters.binary_search_by(|probe| probe.cmp(&seek));

        let mut device = Device {
            name: name.clone(),
            queue: Queue { session: session },
        };
        device.configure(&config)?;
        Ok(device)
    }

    /// Return whether the device has packet information
    pub fn has_packet_information(&mut self) -> bool {
        false
    }
    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.queue.set_nonblock()
    }

    pub fn try_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queue.try_read(buf)
    }
}

impl TryRead for Device {
    fn try_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queue.try_read(buf)
    }
}

impl TryWrite for Device {
    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queue.try_write(buf)
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
        self.name.as_str()
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
        Ok(1504)
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
}

impl Queue {
    pub fn has_packet_information(&mut self) -> bool {
        false
    }

    pub fn set_nonblock(&self) -> io::Result<()> {
        Ok(())
    }
}

impl TryRead for Queue {
    fn try_read(&mut self,  buf: &mut [u8]) -> io::Result<usize> {
        let reader_session = self.session.clone();
        // let mut buffer = [0; 10];
         let    buffer2 = & b"aaaaaa".clone()[..];
        // let mut buffer3 =  &buffer2[..];
        // let  ref mut  str_slice =  ["one", "two", "three"];

        let mut b = buf;
        match reader_session.try_receive() {
            Ok(op) => match op {
                None => Ok(0),
                Some(mut packet) =>{
                    io::copy(&mut packet.bytes(),    &mut b);
                    Ok(packet.bytes().len())
                }
            }
            Err(_) => Err(io::Error::from(io::ErrorKind::Other))
        }
    }
}

impl TryWrite for Queue {
    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write(buf)
    }
}

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let reader_session = self.session.clone();
        let pkt = reader_session.receive_blocking();
        return match pkt {
            Ok(pkt) => {
                let d = pkt.bytes();
                Ok(d.len())
            }
            Err(e) => {
                Ok(0)
            }
        };
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        let mut s: usize = 0;
        for buf in bufs {
            let a = self.read(buf).unwrap();
            s += a;
        }
        Ok(s)
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        let writer_session = self.session.clone();
        let mut packet = writer_session.allocate_send_packet(size as u16).unwrap();
        let mut b = buf;
        io::copy(&mut b, &mut packet.bytes_mut());
        // packet::buffer::Slice::new(packet.bytes_mut());
        writer_session.send_packet(packet);
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let mut sent: usize = 0;
        for buf in bufs {
            self.write(buf);
            sent += buf.len();
        }
        Ok(sent)
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.session.shutdown();
    }
}
