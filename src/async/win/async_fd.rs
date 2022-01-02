use std::io;
use crate::{create, Device};
use std::task::{Poll, Context};
use crate::platform::windows::{TryRead, TryWrite};

pub(crate) struct AsyncFd<T: TryRead + TryWrite> {
    inner: T,
}

impl<T: TryRead + TryWrite> AsyncFd<T> {
    pub fn new(inner: T) -> io::Result<Self> {
        Ok(AsyncFd {
            inner,
        })
    }

    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the backing object of this [`AsyncFd`].
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn try_write(&mut self, buf: & [u8]) -> io::Result<usize> {
        self.inner.try_write(buf)
    }

    pub fn try_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.try_read(buf)
    }
}