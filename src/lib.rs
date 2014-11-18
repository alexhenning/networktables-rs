#![feature(if_let)]

pub use self::client::{Client, Get, Set};
pub use self::errors::{NtResult, NtError, NtErrorKind, UnsupportedType, StringConversionError,
                       KeyAlreadyExists, IdAlreadyExists, IdDoesntExist, OutOfOrderSequenceNumbers,
                       NetworkProblem,};
pub use sequence_numbers::SequenceNumber;

mod client;
mod server;
mod protocol;
mod sequence_numbers;
mod errors;

