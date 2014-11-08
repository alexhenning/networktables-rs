
use super::protocol;
use super::NtResult;
use super::{NtError, NetworkProblem};

use std::sync::{Arc, Mutex};

use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::IoError;
use std::io::IoResult;
use std::str;

use std::kinds::Sync;

#[deriving(Sync)]
pub struct Client {
	address: &'static str,
	// entriesByName: map[string]entry
	// entriesByID:   map[uint16]entry
	// toSend:        map[string]entry
	// done:          chan error
	// state: State,
	connection: Mutex<TcpStream>,
	// writeM:        sync.Mutex
	// m:             sync.Mutex
}

pub enum State {
    Connected,
    Disconnected,
    Error(IoError)
}

// impl std::kinds::Sync for Client {}
// impl Sync for Client {}

impl Client {
    pub fn new(address: &'static str) -> NtResult<Arc<Client>> {
        let mut connection = Mutex::new(try!(connect(address)));
        {
            // Make sure to release the lock
            try!(protocol::hello(&mut *connection.lock()))
        }
        
        let client = Arc::new(Client{address: address, connection: connection});
        let client2 = client.clone();
        spawn(proc() client2.listen());
        Ok(client)
        // Err(NtError{kind: InvalidAddress})
    }

    fn listen(&self) {
        let mut connection = {
            let mutex = self.connection.lock();
            (*mutex).clone()
        };

        // TODO: Read bytes
        loop {
            let msg = match connection.read_u8() {
                Ok(b) => b,
                Err(e) => panic!(e),
            };
            match msg {
                protocol::HELLO_COMPLETE => println!("Hello completed successfully."),
                protocol::ENTRY_ASSIGNMENT => {
                    println!("{}", protocol::parse_assignment(&mut connection));
                },
                m => println!("Unsupported message type 0x{:02X}", m), //panic!(format!("Unsupported message type {}", m)),
            }
        }
    }
    
    // pub fn connect_and_listen(&mut self) {
    //     self.connection = match TcpStream::connect(self.address) {
    //         Ok(stream) => Some(stream),
    //         Err(err) => {
    //             println!("networktables: {}", err);
    //             self.state = Error(err);
    //             return
    //         },
    //     };
        
    //     println!("Got connection.");
    // }
}

pub fn connect(address: &'static str) -> NtResult<TcpStream> {
    let connection = try!(TcpStream::connect(address));
    println!("Got connection.");
    Ok(connection)
}
