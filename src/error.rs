use std::error::Error;

use parking_lot::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Could not find app data dir")]
    UnavailableConfigDir,
    #[error("An error occured during crypto operatation")]
    CryptographyError,
}

impl From<ring::error::Unspecified> for ApplicationError {
    fn from(_: ring::error::Unspecified) -> Self {
        Self::CryptographyError
    }
}

#[derive(Clone)]
pub struct ArcAnyhowError(std::sync::Arc<anyhow::Error>);

impl ArcAnyhowError {
    pub fn new(err: anyhow::Error) -> Self {
        Self(std::sync::Arc::new(err))
    }
}

impl std::fmt::Debug for ArcAnyhowError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ArcAnyhowError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ArcAnyhowError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl From<&mut ArcAnyhowError> for anyhow::Error {
    fn from(err: &mut ArcAnyhowError) -> Self {
        anyhow::Error::new(err.clone())
    }
}

#[derive(Debug)]
pub struct RouilleError {
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
pub struct SyncError<T: Error> {
    lock: Mutex<T>,
}

impl<T: Error> SyncError<T> {
    pub fn new(error: T) -> Self {
        SyncError {
            lock: Mutex::new(error),
        }
    }

    #[allow(unused)]
    pub fn into_inner(self) -> T {
        self.lock.into_inner()
    }

    #[allow(unused)]
    pub fn get_mut(&mut self) -> &mut T {
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
