// #[macro_use]
// extern crate log;

#[macro_use]
extern crate wrapped_string_type_macro;

mod client;
mod data;
mod error;
mod locker;

pub use client::*;
pub use data::*;
pub use error::*;
pub use locker::*;

pub type LockerResult<T> = Result<T, LockerError>;
