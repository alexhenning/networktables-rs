
use super::{NtResult, NtError, StringConversionError, UnsupportedType, IdDoesntExist};
pub use super::sequence_numbers::SequenceNumber;

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

/// Tests
#[cfg(test)]
mod test {
    use super::{Entry, Boolean, Number, String};
    use super::SequenceNumber;
    
    #[test]
    fn entry_basics() {
        let eb = Entry{name: "Boolean".into_string(),
                       id: 0u16, sequence: SequenceNumber(0u16), value: Boolean(true)};
        assert_eq!("Boolean", eb.name.as_slice());
        assert_eq!(0u16, eb.id);
        assert_eq!(SequenceNumber(0u16), eb.sequence);
        assert_eq!(true, match eb.value {
            Boolean(b) => b,
            _ => false,
        });
        
        let ne = Entry{name: "Number".into_string(),
                       id: 1u16, sequence: SequenceNumber(0u16), value: Number(42f64)};
        assert_eq!("Number", ne.name.as_slice());
        assert_eq!(1u16, ne.id);
        assert_eq!(SequenceNumber(0u16), ne.sequence);
        assert_eq!(42f64, match ne.value {
            Number(n) => n,
            _ => 0f64,
        });
        
        let se = Entry{name: "String".into_string(),
                       id: 2u16, sequence: SequenceNumber(0u16),
                       value: String("Test".into_string())};
        assert_eq!("String", se.name.as_slice());
        assert_eq!(2u16, se.id);
        assert_eq!(SequenceNumber(0u16), se.sequence);
        assert_eq!("Test", match se.value {
            String(s) => s,
            _ => "Not a string".into_string(),
        }.as_slice());
    }
}
