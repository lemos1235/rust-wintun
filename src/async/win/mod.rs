use std::io;
use crate::Device;

pub struct AsyncFd<T: Device> {
    inner: T,
}

impl <T> AsyncFd<T> {
    pub fn new(inner: T) -> io::Result<Self> {
        // Self::with_interest(inner, ALL_INTEREST)
        Ok(AsyncFd {
            inner: inner
        })
    }
}