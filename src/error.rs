use crate::LockKey;
use std::error::Error;

#[derive(Debug)]
pub enum LockerError {
    Timeout,
    AlreadyLocked(LockKey),
    AlreadyUnlocked(LockKey),
    MaybeAlreadyUnlocked(LockKey),
    AccessError(String),
}

impl std::fmt::Display for LockerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl Error for LockerError {}
