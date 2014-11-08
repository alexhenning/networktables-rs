
pub use client::Client;

use std::error::Error;
use std::error::FromError;
use std::io::IoError;

pub mod client;
pub mod server;
mod protocol;
    
// pub fn connect_and_listen(address: &'static str) -> NtResult<Client> {
//     let client = try!(Client::new(address));
    // let mut client = match maybe_client {
    //     Ok(c) => c,
    //     Err => return None,
    // };
    // spawn(proc() client.connect_and_listen());
    // Some(client)
// }

pub type NtResult<T> = Result<T, NtError>;

#[deriving(Show)]
pub enum NtErrorKind {
    StringConversionError,
    NetworkProblem(IoError),
}

#[deriving(Show)]
pub struct NtError {
    pub kind: NtErrorKind,
    // pub desc: &'static str,
    // pub detail: Option<String>,
    // pub cause: Option<Error>,
}

impl Error for NtError {
    fn description(&self) -> &str {
        match self.kind {
            StringConversionError => "Error parsing string.",
            NetworkProblem(_) => "Problem connecting to server."
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            NetworkProblem(ref err) => Some(&*err as &Error),
            _ => None,
        }
    }
}

impl FromError<IoError> for NtError {
    fn from_error(err: IoError) -> NtError {
        // TODO: Generalize and maintain cause
        NtError{kind: NetworkProblem(err)}
    }
}
