pub use self::client::{Client, Get, Set};
pub use self::errors::{NtResult, NtError, NtErrorKind, UnsupportedType, StringConversionError,
                       KeyAlreadyExists, IdAlreadyExists, IdDoesntExist, NetworkProblem,};


mod client;
mod server;
mod protocol;
mod errors;
    
// pub fn connect_and_listen(address: &'static str) -> NtResult<Client> {
//     let client = try!(Client::new(address));
    // let mut client = match maybe_client {
    //     Ok(c) => c,
    //     Err => return None,
    // };
    // spawn(proc() client.connect_and_listen());
    // Some(client)
// }


