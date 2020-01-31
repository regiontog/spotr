use std::error::Error;

use parking_lot::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Could not find app data dir")]
    UnavailableConfigDir,
    #[error("An error occured during crypto operatation")]
    CryptographyError(ring::error::Unspecified),
}

impl From<ring::error::Unspecified> for ApplicationError {
    fn from(err: ring::error::Unspecified) -> Self {
        Self::CryptographyError(err)
    }
}

#[derive(Debug)]
pub(super) struct RouilleError {
    inner: Box<dyn Error + Send + Sync>,
}

impl From<Box<dyn Error + Send + Sync>> for RouilleError {
    fn from(inner: Box<dyn Error + Send + Sync>) -> Self {
        Self { inner }
    }
}

impl std::fmt::Display for RouilleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl Error for RouilleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source()
    }

    // fn backtrace(&self) -> Option<&Backtrace> {
    //     self.inner.backtrace()
    // }
}

#[derive(Debug)]
pub(super) struct SyncError<T: Error> {
    lock: Mutex<T>,
}

impl<T: Error> SyncError<T> {
    pub(super) fn new(error: T) -> Self {
        SyncError {
            lock: Mutex::new(error),
        }
    }

    pub(super) fn into_inner(self) -> T {
        self.lock.into_inner()
    }

    pub(super) fn get_mut(&mut self) -> &mut T {
        self.lock.get_mut()
    }
}

impl<T: Error> std::fmt::Display for SyncError<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.lock.lock().fmt(formatter)
    }
}

impl<T: Error> Error for SyncError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // FIXME: Somehow provide this and release the lock?
        None
    }

    // fn backtrace(&self) -> Option<&Backtrace> {
    //     self.lock.lock().backtrace()
    // }
}
