
use super::protocol;
use super::NtResult;
use super::{NtError, KeyAlreadyExists, IdAlreadyExists};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use std::io::Listener;

use std::io::net::tcp::TcpStream;
use std::io::IoError;

pub trait Get<T> {
    fn get(&self, key: String) -> Option<T>;
}

// pub trait SubTable : Get<bool> + Get<f64> + Get<String> {}

#[deriving(Sync)]
pub struct Client {
    entries_by_name: Mutex<HashMap<String, protocol::Entry>>,
    entries_by_id: Mutex<HashMap<u16, protocol::Entry>>,
	// toSend:        map[string]entry
	// done:          chan error // state: State,
	connection: Mutex<TcpStream>,
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
        let connection = Mutex::new(try!(connect(address)));
        {
            // Make sure to release the lock
            try!(protocol::hello(&mut *connection.lock()))
        }
        
        let client = Arc::new(Client{
            entries_by_name: Mutex::new(HashMap::new()),
            entries_by_id: Mutex::new(HashMap::new()),
            connection: connection
        });
        
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

        loop {
            let msg = match connection.read_u8() {
                Ok(b) => b,
                Err(e) => panic!(e),
            };
            match msg {
                protocol::HELLO_COMPLETE => self.handle_hello_complete(),
                protocol::ENTRY_ASSIGNMENT => self.handle_entry_assignment(&mut connection.clone()),
                protocol::ENTRY_UPDATE => self.handle_entry_update(&mut connection.clone()),
                m => println!("Unsupported message type 0x{:02X}", m), // panic!(format!("Unsupported message type {}", m)), // TODO: Handle more gracefully
            }
        }
    }

    fn handle_hello_complete(&self) {
        println!("Hello completed successfully.");
    }

    fn handle_entry_assignment(&self, connection: &mut TcpStream) {
        match self.assign_entry(match protocol::parse_assignment(connection) {
            Ok(e) => e,
            Err(e) => panic!("Error parsing assignment: {}", e), // TODO: Handle more gracefully
        }) {
            Ok(_) => (),
            Err(e) => panic!("Error assigning entry: {}", e), // TODO: Handle more gracefully
        }
    }

    fn handle_entry_update(&self, connection: &mut TcpStream) {
        match self.assign_entry(match protocol::parse_update(connection, |id| self.id_lookup(id)) {
            Ok(e) => e,
            Err(e) => panic!("Error parsing update: {}", e), // TODO: Handle more gracefully
        }) {
            Ok(_) => (),
            Err(e) => panic!("Error updating entry: {}", e), // TODO: Handle more gracefully
        }
    }

    fn assign_entry(&self, entry: protocol::Entry) -> NtResult<()> {
        println!("{}", entry);
        
        let mut names = self.entries_by_name.lock();
        if names.contains_key(&entry.name) {
            return Err(NtError{kind: KeyAlreadyExists(entry.name)})
        }

        let mut ids = self.entries_by_id.lock();
        if ids.contains_key(&entry.id) {
            return Err(NtError{kind: IdAlreadyExists(entry.id)})
        }

        let name = entry.name.clone();
        names.insert(name, entry.clone());
        let id = entry.id.clone();
        ids.insert(id, entry);
        
        Ok(())
    }

    fn get_entry(&self, key: String) -> Option<protocol::EntryType> {
        let names = self.entries_by_name.lock();
        match names.get(&key) {
            Some(entry) => {
                Some((*entry).value.clone())
            },
            None => None,
        }
    }

    fn id_lookup(&self, id: u16) -> Option<(String, protocol::EntryType)> {
        let ids = self.entries_by_id.lock();
        let entry = match ids.get(&id) {
            Some(entry) => entry.clone(),
            None => return None,
        };
        Some((entry.name, entry.value))
    }
}

impl Get<bool> for Client {
    fn get(&self, key: String) -> Option<bool> {
        match self.get_entry(key) {
            Some(protocol::Boolean(b)) => Some(b),
            _ => None,
        }
    }
}

impl Get<f64> for Client {
    fn get(&self, key: String) -> Option<f64> {
        match self.get_entry(key) {
            Some(protocol::Number(n)) => Some(n),
            _ => None,
        }
    }
}

impl Get<String> for Client {
    fn get(&self, key: String) -> Option<String> {
        match self.get_entry(key) {
            Some(protocol::String(s)) => Some(s),
            _ => None,
        }
    }
}

fn connect(address: &'static str) -> NtResult<TcpStream> {
    let connection = try!(TcpStream::connect(address));
    println!("Got connection.");
    Ok(connection)
}

