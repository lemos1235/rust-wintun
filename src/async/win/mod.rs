mod async_fd;

use async_fd::*;

pub(crate) mod device;

pub(crate) use device::{AsyncDevice2 as AsyncDevice, AsyncQueue2 as AsyncQueue};