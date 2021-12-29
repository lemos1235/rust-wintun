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
use std::io::{self, Read, Write};
use std::mem;
use std::net::Ipv4Addr;
// use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::ptr;
use std::sync::Arc;
use std::vec::Vec;
// use futures::future::ok;

use libc;
use libc::{c_char, c_short};
// use libc::{AF_INET, O_RDWR, SOCK_DGRAM};

use crate::configuration::{Configuration, Layer};
use crate::device::Device as D;
use crate::error::*;
// use crate::platform::linux::sys::*;
// use crate::platform::posix::{Fd, SockAddr};

/// A TUN device using the wintun driver.
pub struct Device {
    name: String,
    queues: Vec<Queue>,
    // ctl: Fd,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let wintun = unsafe { wintun::load_from_path("wintun.dll") }
            .expect("Failed to load wintun dll");

        // let dev = match config.name.as_ref() {
        //             Some(name) => {
        //                 let name = CString::new(name.clone())?;
        //
        //                 if name.as_bytes_with_nul().len() > IFNAMSIZ {
        //                     return Err(Error::NameTooLong);
        //                 }
        //
        //                 Some(name)
        //             }
        //
        //             None => None,
        //         };
        //
        // let adapter = match wintun::Adapter::open(&wintun, config.name.unwrap().as_str()) {
        //     Ok(a) => a,
        //     Err(_) => wintun::Adapter::create(&wintun, "Example", "Demo", None)
        //         .expect("Failed to create wintun adapter!"),
        // };
        // let session = Arc::new(adapter.start_session(wintun::MAX_RING_CAPACITY).unwrap());
        // let reader_session = session.clone();
        // session.shutdown();

        // let mut device = unsafe {
        //     let dev = match config.name.as_ref() {
        //         Some(name) => {
        //             let name = CString::new(name.clone())?;
        //
        //             if name.as_bytes_with_nul().len() > IFNAMSIZ {
        //                 return Err(Error::NameTooLong);
        //             }
        //
        //             Some(name)
        //         }
        //
        //         None => None,
        //     };
        //
        //     let mut queues = Vec::new();
        //
        //     let mut req: ifreq = mem::zeroed();
        //
        //     if let Some(dev) = dev.as_ref() {
        //         ptr::copy_nonoverlapping(
        //             dev.as_ptr() as *const c_char,
        //             req.ifrn.name.as_mut_ptr(),
        //             dev.as_bytes().len(),
        //         );
        //     }
        //
        //     let device_type: c_short = config.layer.unwrap_or(Layer::L3).into();
        //
        //     let queues_num = config.queues.unwrap_or(1);
        //     if queues_num < 1 {
        //         return Err(Error::InvalidQueuesNumber);
        //     }
        //
        //     req.ifru.flags = device_type
        //         | if config.platform.packet_information {
        //             0
        //         } else {
        //             IFF_NO_PI
        //         }
        //         | if queues_num > 1 { IFF_MULTI_QUEUE } else { 0 };
        //
        //     for _ in 0..queues_num {
        //         let tun = Fd::new(libc::open(b"/dev/net/tun\0".as_ptr() as *const _, O_RDWR))
        //             .map_err(|_| io::Error::last_os_error())?;
        //
        //         if tunsetiff(tun.0, &mut req as *mut _ as *mut _) < 0 {
        //             return Err(io::Error::last_os_error().into());
        //         }
        //
        //         queues.push(Queue {
        //             tun,
        //             pi_enabled: config.platform.packet_information,
        //         });
        //     }



            // let ctl = Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0))
            //     .map_err(|_| io::Error::last_os_error())?;
            //
            // Device {
            //     name: CStr::from_ptr(req.ifrn.name.as_ptr())
            //         .to_string_lossy()
            //         .into(),
            //     queues: queues,
            //     // ctl: ctl,
            // }
        // };

        // device.configure(&config)?;

        // Ok(device);
        Ok(Device { name: "".to_string(), queues: vec![] })
    }

    /// Prepare a new request.
    // unsafe fn request(&self) -> ifreq {
    //     let mut req: ifreq = mem::zeroed();
    //     ptr::copy_nonoverlapping(
    //         self.name.as_ptr() as *const c_char,
    //         req.ifrn.name.as_mut_ptr(),
    //         self.name.len(),
    //     );
    //
    //     req
    // }

    /// Make the device persistent.
    pub fn persist(&mut self) -> Result<()> {
        // unsafe {
        //     if tunsetpersist(self.as_raw_fd(), &1) < 0 {
        //         Err(io::Error::last_os_error().into())
        //     } else {
        //         Ok(())
        //     }
        // }
        Ok(())
    }

    /// Set the owner of the device.
    pub fn user(&mut self, value: i32) -> Result<()> {
        // unsafe {
        //     if tunsetowner(self.as_raw_fd(), &value) < 0 {
        //         Err(io::Error::last_os_error().into())
        //     } else {
        //         Ok(())
        //     }
        // }
        Ok(())
    }

    /// Set the group of the device.
    pub fn group(&mut self, value: i32) -> Result<()> {
        // unsafe {
        //     if tunsetgroup(self.as_raw_fd(), &value) < 0 {
        //         Err(io::Error::last_os_error().into())
        //     } else {
        //         Ok(())
        //     }
        // }
        Ok(())
    }

    /// Return whether the device has packet information
    pub fn has_packet_information(&mut self) -> bool {
        self.queues[0].has_packet_information()
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.queues[0].set_nonblock()
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queues[0].read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.queues[0].read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queues[0].write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.queues[0].flush()
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.queues[0].write_vectored(bufs)
    }
}

impl D for Device {
    type Queue = Queue;

    fn name(&self) -> &str {
        return "";
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
        Err(Error::NotImplemented)
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        Ok(())
    }

    fn queue(&mut self, index: usize) -> Option<&mut Self::Queue> {
        // if index > 0 {
        //     return None;
        // }
        //
        // Some(&mut self.queue)

        return None;
    }
}

pub struct Queue {
    // tun: Fd,
    pi_enabled: bool,
}

impl Queue {
    pub fn has_packet_information(&mut self) -> bool {
        self.pi_enabled
    }

    pub fn set_nonblock(&self) -> io::Result<()> {
        // self.tun.set_nonblock()
        Ok(())
    }
}

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // self.tun.read(buf)
        Ok(0)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        // self.tun.read_vectored(bufs)
        Ok(0)
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // self.tun.write(buf)
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        // self.tun.flush()
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        // self.tun.write_vectored(bufs)
        Ok(0)
    }
}

impl Into<c_short> for Layer {
    fn into(self) -> c_short {
        // match self {
        //     Layer::L2 => IFF_TAP,
        //     Layer::L3 => IFF_TUN,
        // }
        0
    }
}
