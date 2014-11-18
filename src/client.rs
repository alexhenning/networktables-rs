use super::protocol;
use super::NtResult;
use super::{NtError, KeyAlreadyExists, IdAlreadyExists, OutOfOrderSequenceNumbers,  NetworkProblem};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use std::io::Listener;
use std::io::net::tcp::TcpStream;
use std::io::Timer;
use std::time::Duration;

/// A trait for getting values of different types by a key.
pub trait Get<T> {
    /// Returns the value of `Some(T)` if it exists in the tabel and
    /// is coercible into the desired value.
    fn get(&self, key: String) -> Option<T>;
}

/// A trait for setting values of different types by a key.
pub trait Set<T> {
    /// Returns the value of `Some(T)` if it exists in the tabel and
    /// is coercible into the desired value.
    fn set(&self, key: String, value: T) -> NtResult<()>;
}

// TODO: Figure out table trait
// pub trait Table : Get<bool> + Get<f64> + Get<String> {}
// pub trait Table : Get<bool + f64 + String> {}

// TODO: better map without race conditions
// Locking order to avoid deadlocks:
// - entries_by_name
// - entries_by_id
// - send_queue
// - state
// - connection

/// A [NetworkTables 2.0](https://docs.google.com/document/d/1On9BkUgkmMmTnfVxSQlOZMWsa9Vas6-8cT19TX59Tho/edit)
/// client. It acts as a distributed HashTable that is synchronized
/// with other clients by a central server.
///
/// # Example
///
/// ```ignore
/// extern crate networktables;
/// use networktables;
/// let networktables::Client::new("localhost:1735").unwrap();
/// ```
#[deriving(Sync)]
pub struct Client {
    entries_by_name: Mutex<HashMap<String, protocol::Entry>>,
    entries_by_id: Mutex<HashMap<u16, protocol::Entry>>,
    send_queue: Mutex<Vec<protocol::Entry>>,
    state: Mutex<State>,
    errors: Mutex<Vec<NtError>>,
	connection: Mutex<TcpStream>,
}

/// The state of the clients connection.
#[deriving(PartialEq,Eq,Sync,Clone,Show)]
pub enum State {
    /// The state between starting and receiving th hello complete message.
    Initializing,
    /// The state after receiving a hello complete message.
    Connected,
    /// The state when it has closed down properly.
    Closed,
    /// The state once a fatal error occurs.
    Error(NtError)
}

impl Client {
    pub fn new(address: &'static str) -> NtResult<Arc<Client>> {
        let connection = Mutex::new(try!(TcpStream::connect(address)));
        {   // Make sure the lock is  released
            try!(protocol::write_hello(&mut *connection.lock()))
        }
        
        let client = Arc::new(Client{
            entries_by_name: Mutex::new(HashMap::new()),
            entries_by_id: Mutex::new(HashMap::new()),
            send_queue: Mutex::new(Vec::new()),
            state: Mutex::new(Initializing),
            errors: Mutex::new(Vec::new()),
            connection: connection,
        });
        
        let (client2, client3) = (client.clone(), client.clone());
        spawn(proc() client2.listen());
        spawn(proc() client3.send());

        // TODO: Block until initialized?
        Ok(client)
    }

    pub fn close(&self) {
        let mut state = self.state.lock();
        match *state {
            Initializing | Connected => { *state = Closed; },
            Closed => return,
            Error(_) => (),
        }
        
        let mut connection = self.clone_connection();
        if let Err(e) = connection.close_read() { println!("{}", e) };
        if let Err(e) = connection.close_write() { println!("{}", e) };
    }

    pub fn get_state(&self) -> State { self.state.lock().clone() }
    pub fn get_errors(&self) -> Vec<NtError> { self.errors.lock().clone() }
    fn clone_connection(&self) -> TcpStream { self.connection.lock().clone() }

    fn send(&self) {
        let keep_alive_cutoff: u64 = 1000 /*ms*/ / 20 /*ms*/;
        let mut counter = 0;
        let mut timer = Timer::new().unwrap(); // TODO: Possibility for panic?
        let periodic = timer.periodic(Duration::milliseconds(20));

        loop {
            periodic.recv();
            if let Err(e) = self.send_queue() {
                return self.log_fatal(e)
            }
            
            counter += 1;
            if (counter % keep_alive_cutoff) == 0 {
                counter = 0;
                if let Err(e) = self.send_keep_alive() {
                    return self.log_fatal(e)
                }
            }
        }
    }

    fn send_queue(&self) -> NtResult<()> {
        let mut connection = self.clone_connection();

        // Send all entries in the queue
        let mut queue = self.send_queue.lock();
        for entry in queue.iter() {
            try!(match entry.id.clone() {
                protocol::CLIENT_REQUEST_ID => protocol::write_assignment(&mut connection, entry),
                _ => protocol::write_update(&mut connection, entry),
            });
        }

        // Clear queue
        *queue = Vec::new();
        Ok(())
    }

    fn send_keep_alive(&self) -> NtResult<()> {
        let mut connection = self.clone_connection();
        protocol::write_keep_alive(&mut connection)
    }
    
    fn listen(&self) {
        let mut connection = self.clone_connection();

        loop {
            let msg = match connection.read_u8() {
                Ok(b) => b,
                Err(e) => return self.log_fatal(NtError{kind: NetworkProblem(e)}),
            };
            match msg {
                protocol::HELLO_COMPLETE => self.handle_hello_complete(),
                protocol::ENTRY_ASSIGNMENT => self.handle_entry_assignment(),
                protocol::ENTRY_UPDATE => self.handle_entry_update(),
                m => println!("Unsupported message type 0x{:02X}", m), // panic!(format!("Unsupported message type {}", m)), // TODO: Handle more gracefully
            }
        }
    }

    fn handle_hello_complete(&self) {
        let mut state = self.state.lock();
        if *state == Initializing {
            *state = Connected;
        }
    }

    fn handle_entry_assignment(&self) {
        let mut connection = self.clone_connection();
        let entry = match protocol::parse_assignment(&mut connection) {
            Ok(e) => e,
            Err(e) => return self.log_fatal(e),
        };
        
        let mut names = self.entries_by_name.lock();
        if names.contains_key(&entry.name) {
            self.log_error(NtError{kind: KeyAlreadyExists(entry.name)});
            return
        }

        let mut ids = self.entries_by_id.lock();
        if ids.contains_key(&entry.id) {
            self.log_error(NtError{kind: IdAlreadyExists(entry.id)});
            return
        }

        let (name, id) = (entry.name.clone(), entry.id.clone());
        names.insert(name, entry.clone());
        ids.insert(id, entry);
    }

    fn handle_entry_update(&self) {
        let mut connection = self.clone_connection();
        let entry = match protocol::parse_update(&mut connection, |id| self.id_lookup(id)) {
            Ok(e) => e,
            Err(e) => return self.log_fatal(e)
        };
        
        let mut names = self.entries_by_name.lock();
        let mut ids = self.entries_by_id.lock();

        // Test sequence numbers
        let name = entry.name.clone();
        {
            // Limit the scope of borrowing
            let old_entry = match names.get(&name) {
                Some(e) => e,
                None => // TODO: Handle more gracefully? Name must exist to update unless entry got corrupted.
                    panic!("No entry exists to update with name={}", entry.name),
            };
            if old_entry.sequence >= entry.sequence {
                self.log_error(NtError{kind: OutOfOrderSequenceNumbers(old_entry.sequence, entry.sequence)});
                return
            }
        }

        names.insert(name, entry.clone());
        let id = entry.id.clone();
        ids.insert(id, entry);
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
    
    fn set_entry(&self, key: String, value: protocol::EntryType) -> NtResult<()> {
        let names = self.entries_by_name.lock();
        let mut queue = self.send_queue.lock();
        let mut entry = match names.get(&key) {
            Some(entry) => entry.clone(),
                // TODO: Assert that values have the same type or Err(NtError{kind: ???})
            None => protocol::Entry{
                name: key,
                id: protocol::CLIENT_REQUEST_ID,
                sequence: protocol::SequenceNumber(0u16),
                value: protocol::Boolean(false),
            },
        };
        
        entry.value = value;
        entry.sequence.increment();
        queue.push(entry);
        Ok(())
    }

    fn id_lookup(&self, id: u16) -> Option<(String, protocol::EntryType)> {
        let ids = self.entries_by_id.lock();
        let entry = match ids.get(&id) {
            Some(entry) => entry.clone(),
            None => return None,
        };
        Some((entry.name, entry.value))
    }

    fn log_fatal(&self, err: NtError) {
        match self.get_state() {
            Closed | Error(_) => self.log_error(err),
            Initializing | Connected => {
                let mut state = self.state.lock();
                *state = Error(err);
            }
        }
    }
    
    fn log_error(&self, err: NtError) {
        let mut errors = self.errors.lock();
        errors.push(err);
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

impl Set<bool> for Client {
    fn set(&self, key: String, value: bool) -> NtResult<()> {
        self.set_entry(key, protocol::Boolean(value))
    }
}

impl Set<f64> for Client {
    fn set(&self, key: String, value: f64) -> NtResult<()> {
        self.set_entry(key, protocol::Number(value))
    }
}

impl Set<String> for Client {
    fn set(&self, key: String, value: String) -> NtResult<()> {
        self.set_entry(key, protocol::String(value))
    }
}

