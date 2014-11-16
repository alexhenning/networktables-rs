
use super::{NtResult, NtError, StringConversionError, UnsupportedType, IdDoesntExist};

/// Protocol constants

// The version of the protocol currently implemented.
const VERSION: u16 = 0x0200;

// ClientRequestID is the id clients use when requesting the server
// assign an id to the key.
pub const CLIENT_REQUEST_ID: u16 = 0xFFFF;

// Values used to indicate the various message types used in the
// NetworkTables protocol.
pub const KEEP_ALIVE: u8 = 0x00;
pub const HELLO: u8 = 0x01;
// pub const VERSION_UNSUPPORTED: u8 = 0x02;
pub const HELLO_COMPLETE: u8 = 0x03;
pub const ENTRY_ASSIGNMENT: u8 = 0x10;
pub const ENTRY_UPDATE: u8 = 0x11;

// Types of data that can be sent over NetworkTables.s
const TYPE_BOOLEAN: u8 = 0x00;
const TYPE_NUMBER: u8 = 0x01;
const TYPE_STRING: u8 = 0x02;
// const TYPE_BOOLEAN_ARRAY: u8 = 0x10;
// const TYPE_DOUBLE_ARRAY: u8 = 0x11;
// const TYPE_STRING_ARRAY: u8 = 0x12;


/// Entry definition
#[deriving(Show, Clone)]
pub struct Entry {
    pub name: StdString,
    pub id: u16,
    pub sequence: SequenceNumber,
    pub value: EntryType,
}

// Since we overloaded the name string, maybe we should have NtString
// instead. We'll see what makes sense.
type StdString = ::std::string::String;

#[deriving(Show, Clone)]
pub enum EntryType {
    Boolean(bool),
    Number(f64),
    String(StdString),
}

/// Protocol utilities
pub fn write_hello<T: Writer>(w: &mut T) -> NtResult<()> {
    try!(w.write_u8(HELLO));
    Ok(try!(w.write_be_u16(VERSION)))
}

pub fn write_keep_alive<T: Writer>(w: &mut T) -> NtResult<()> {
    Ok(try!(w.write_u8(KEEP_ALIVE)))
}

pub fn write_assignment<T: Writer>(w: &mut T, entry: &Entry) -> NtResult<()> {
    try!(w.write_u8(ENTRY_ASSIGNMENT));
    try!(write_string(w, entry.name.clone()));
    try!(w.write_u8(match entry.value {
        Boolean(_) => TYPE_BOOLEAN,
        Number(_) => TYPE_NUMBER,
        String(_) => TYPE_STRING,
    }));
    try!(w.write_be_u16(entry.id));
    try!(w.write_be_u16(entry.sequence.as_u16()));
    match entry.value {
        Boolean(b) => try!(w.write_u8(match b {true => 0x01u8, false => 0x00u8})),
        Number(n) => try!(w.write_be_f64(n)),
        String(ref s) => try!(write_string(w, s.clone())),
    };
    Ok(())
}

pub fn parse_assignment<T: Reader>(r: &mut T) -> NtResult<Entry> {
    let name = try!(parse_string(r));
    let typ = try!(r.read_u8());
    let id = try!(r.read_be_u16());
    let seq_number = SequenceNumber(try!(r.read_be_u16()));
    let value = match typ {
        TYPE_BOOLEAN => Boolean(try!(r.read_u8()) != 0u8),
        TYPE_NUMBER => Number(try!(r.read_be_f64())),
        TYPE_STRING => String(try!(parse_string(r))),
        t => return Err(NtError{kind: UnsupportedType(t)}),
    };
    Ok(Entry{name: name, id: id, sequence: seq_number, value: value})
}

pub fn write_update<T: Writer>(w: &mut T, entry: &Entry) -> NtResult<()> {
    try!(w.write_u8(ENTRY_UPDATE));
    try!(w.write_be_u16(entry.id));
    try!(w.write_be_u16(entry.sequence.as_u16()));
    match entry.value {
        Boolean(b) => try!(w.write_u8(match b {true => 0x01u8, false => 0x00u8})),
        Number(n) => try!(w.write_be_f64(n)),
        String(ref s) => try!(write_string(w, s.clone())),
    };
    Ok(())
}

pub fn parse_update<T: Reader>(r: &mut T, f: |u16| -> Option<(StdString, EntryType)>)
                               -> NtResult<Entry> {
    let id = try!(r.read_be_u16());
    let seq_number = SequenceNumber(try!(r.read_be_u16()));
    let (name, entry_type) = match f(id) {
        Some((name, entry_type)) => (name, entry_type),
        None => return Err(NtError{kind: IdDoesntExist(id)}),
    };
    let value = match entry_type {
        Boolean(_) => Boolean(try!(r.read_u8()) != 0u8),
        Number(_) => Number(try!(r.read_be_f64())),
        String(_) => String(try!(parse_string(r))),
    };
    Ok(Entry{name: name, id: id, sequence: seq_number, value: value})
}

pub fn write_string<T: Writer>(w: &mut T, s: StdString) -> NtResult<()> {
    try!(w.write_be_u16(s.len() as u16));
    for byte in s.into_bytes().iter() {
        try!(w.write_u8(*byte))
    }
    // TODO: Assert number of bits written == length
    // TODO: Assert that string length is 16 bits
    Ok(())
}

pub fn parse_string<T: Reader>(r: &mut T) -> NtResult<StdString> {
    let length = try!(r.read_be_u16());
    let vec = try!(r.read_exact(length as uint));
    match ::std::string::String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(NtError{kind: StringConversionError}),
    }
}

/// Sequence Numbers are a special type of number
/// Implements [rfc1982](http://tools.ietf.org/html/rfc1982)
// TODO: Document
#[deriving(Show, Clone)]
pub struct SequenceNumber(pub u16);
static SEQUENCE_NUMBER_DIVIDING_POINT: u16 = 32768u16;

impl SequenceNumber {
    pub fn increment(&mut self) {
        let SequenceNumber(n) = *self;
        *self = SequenceNumber(n + 1);
    }
    
    pub fn as_u16(&self) -> u16 {
        let SequenceNumber(n) = *self;
        n
    }
    
    fn cmp(&self, other: &SequenceNumber) -> Ordering {
        let s = match *self { SequenceNumber(n) => n };
        let o = match *other { SequenceNumber(n) => n };

        if s == o {
            Equal
        } else if (s < o && o-s < SEQUENCE_NUMBER_DIVIDING_POINT)
            || (s > o && s-o > SEQUENCE_NUMBER_DIVIDING_POINT) {
                Less
        } else {
            Greater
        }
    }
}

impl PartialEq for SequenceNumber {
    fn eq(&self, other: &SequenceNumber) -> bool {
        self.cmp(other) == Equal
    }
}
    
impl Eq for SequenceNumber {}

impl PartialOrd for SequenceNumber {
    fn partial_cmp(&self, other: &SequenceNumber) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Tests
#[cfg(test)]
mod test {
    use super::{Entry, Boolean, Number, String};
    use super::{SequenceNumber, SEQUENCE_NUMBER_DIVIDING_POINT};
    use std::rand;
    
    #[test]
    fn entry_basics() {
        let eb = Entry{name: StdString::from_str("Boolean"),
                       id: 0u16, sequence: SequenceNumber(0u16), value: Boolean(true)};
        assert_eq!("Boolean", eb.name.as_slice());
        assert_eq!(0u16, eb.id);
        assert_eq!(SequenceNumber(0u16), eb.sequence);
        assert_eq!(true, match eb.entry {
            Boolean(b) => b,
            _ => false,
        });
        
        let ne = Entry{name: StdString::from_str("Number"),
                       id: 1u16, sequence: SequenceNumber(0u16), value: Number(42f64)};
        assert_eq!("Number", ne.name.as_slice());
        assert_eq!(1u16, ne.id);
        assert_eq!(SequenceNumber(0u16), ne.sequence);
        assert_eq!(42f64, match ne.entry {
            Number(n) => n,
            _ => 0f64,
        });
        
        let se = Entry{name: StdString::from_str("String"),
                       id: 2u16, sequence: SequenceNumber(0u16),
                       value: String(StdString::from_str("Test"))};
        assert_eq!("String", se.name.as_slice());
        assert_eq!(2u16, se.id);
        assert_eq!(SequenceNumber(0u16), se.sequence);
        assert_eq!("Test", match se.entry {
            String(s) => s,
            _ => StdString::from_str(""),
        }.as_slice());
    }

    #[test]
    fn sequence_number_equality() {
        // TODO: ?for n in rand::task_rng().gen_iter::<u16>().take(100) {
        for _ in range::<int>(0, 100) {
            let n = rand::random::<u16>();
            assert_eq!(SequenceNumber(n), SequenceNumber(n));
        }
    }

    #[test]
    fn sequence_number_greater() {
        for _ in range::<int>(0, 100) {
            let n = rand::random::<u16>();
            let i = rand::random::<u16>() % (SEQUENCE_NUMBER_DIVIDING_POINT-1) + 1;
            assert!(SequenceNumber(n) < SequenceNumber(n+i));
        }
    }
}
