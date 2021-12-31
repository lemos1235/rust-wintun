use std::io;
use crate::Device;
use std::task::{Poll, Context};
// use tokio::ready;
// use crate::platform::windows::AsWintun;

pub struct AsyncFd<T> {
    inner: Option<T>,
}

impl <T> AsyncFd<T> {
    pub fn new(inner: T) -> io::Result<Self> {
        Ok(AsyncFd {
            inner: Some(inner),
        })
    }

    #[inline]
    pub fn get_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    /// Returns a mutable reference to the backing object of this [`AsyncFd`].
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    pub fn poll_read_ready_mut<'a>(
        &'a mut self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyMutGuard<'a, T>>> {
        let d = self.get_mut();
        // let event = ready!(self.registration.poll_read_ready(cx))?;

        Ok(AsyncFdReadyMutGuard {
            async_fd: self,
            // event: Some(event),
        })
            .into()
    }

    pub fn poll_write_ready_mut<'a>(
        &'a mut self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<AsyncFdReadyMutGuard<'a, T>>> {
        // let event = ready!(self.registration.poll_write_ready(cx))?;

        Ok(AsyncFdReadyMutGuard {
            async_fd: self,
            // event: Some(event),
        })
            .into()
    }
}

pub struct AsyncFdReadyMutGuard<'a, T> {
    async_fd: &'a mut AsyncFd<T>,
    // event: Option<ReadyEvent>
}

impl<'a, Inner> AsyncFdReadyMutGuard<'a, Inner> {
    pub fn clear_ready(&mut self) {
        // if let Some(event) = self.event.take() {
        //     self.async_fd.registration.clear_readiness(event);
        // }
    }

    pub fn try_io<R>(
        &mut self,
        f: impl FnOnce(&mut AsyncFd<Inner>) -> io::Result<R>,
    ) -> Result<io::Result<R>, TryIoError> {
        let result = f(self.async_fd);

        if let Err(e) = result.as_ref() {
            if e.kind() == io::ErrorKind::WouldBlock {
                self.clear_ready();
            }
        }

        match result {
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Err(TryIoError(())),
            result => Ok(result),
        }
    }
}


#[derive(Debug)]
pub struct TryIoError(());