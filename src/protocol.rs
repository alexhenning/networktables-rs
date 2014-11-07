
/// Protocol constants

// The version of the protocol currently implemented.
static VERSION: u16 = 0x0200;


// ClientRequestID is the id clients use when requesting the server
// assign an id to the key.
static CLIENT_REQUEST_ID: u8 = 0xFFFF;

// Values used to indicate the various message types used in the
// NetworkTables protocol.
// static KEEP_ALIVE: u8 = 0x00;
// static HELLO: u8 = 0x01;
// static VERSION_UNSUPPORTED: u8 = 0x02;
// static HELLO_COMPLETE: u8 = 0x03;
// static ENTRY_ASSIGNMENT: u8 = 0x10;
// static ENTRY_UPDATE: u8 = 0x11;

enum MessagType {
    KeepAlive = 0x00,
    Hello = 0x01,
    VersionUnsupported = 0x02,
    HelloComplete = 0x03,
    EntryAssignment = 0x10,
    EntryUpdate = 0x11,
}

// Types of data that can be sent over NetworkTables.
// static TYPE_BOOLEAN: u8 = 0x00;
// static TYPE_DOUBLE: u8 = 0x01;
// static TYPE_STRING: u8 = 0x02;
// static TYPE_BOOLEAN_ARRAY: u8 = 0x10;
// static TYPE_DOUBLE_ARRAY: u8 = 0x11;
// static TYPE_STRING_ARRAY: u8 = 0x12;

enum Type {
    TBoolean = 0x00,
    TDouble = 0x01,
    TString = 0x02,
    TBooleanArray = 0x10,
    TDoubleArray = 0x11,
    TStringArray = 0x12,
}


/// Entry definition
// TODO: Mutex?
pub struct Entry<'a> {
    name: &'static str,
    id: u16,
    sequence: &'a mut SequenceNumber,
    entry: &'a mut EntryType,
}

pub enum EntryType {
    Boolean(bool),
    Number(f64),
    String(&'static str),
}

impl EntryType {
    pub fn to_bytes(&self) -> Vec<u8> {
        match *self {
            Boolean(b) => get_boolean_bytes(b),
            Number(n) => get_double_bytes(n),
            String(s) => get_string_bytes(s),
        }
    }
    
    pub fn from_bytes() {

    }
}

/// Protocol utilities
fn get_boolean_bytes(val: bool) -> Vec<u8> {
	if val {
		vec![0x01]
	} else {
		vec![0x00]
	}
}

fn get_double_bytes(val: f64) -> Vec<u8> {
	// let bytes = make(&[u8], 8, 8)
	// let bits = math.Float64bits(val)
	// binary.BigEndian.PutUint64(bytes, bits)
	// return bytes
	vec![]
}

fn get_string_bytes(val: &'static str) -> Vec<u8> {
	// bytes := make([]byte, 0, 2+len(val))
	// bytes = append(bytes, getUint16Bytes((uint16)(len(val)))...)
	// bytes = append(bytes, []byte(val)...)
	// return bytes
	vec![]
}



/// Sequence Numbers are a special type of number
/// Implements [rfc1982](http://tools.ietf.org/html/rfc1982)
// TODO: Document
// TODO: Does it need to be a tuple
// TODO: Test
#[deriving(Show)]
pub struct SequenceNumber(u16);
static SEQUENCE_NUMBER_DIVIDING_POINT: u16 = 32768u16;

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

impl Ord for SequenceNumber {
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

/// Tests
mod test {
    use super::{Entry, Boolean, Number, String};
    use super::{SequenceNumber, SEQUENCE_NUMBER_DIVIDING_POINT};
    use std::rand;
    
    #[test]
    fn entry_basics() {
        let eb = Entry{name: "Boolean", id: 0u16, sequence: &mut SequenceNumber(0u16), entry: &mut Boolean(true)};
        assert_eq!("Boolean", eb.name);
        assert_eq!(0u16, eb.id);
        assert_eq!(SequenceNumber(0u16), *eb.sequence);
        assert_eq!(true, match *eb.entry {
            Boolean(b) => b,
            _ => false,
        });
        
        let ne = Entry{name: "Number", id: 1u16, sequence: &mut SequenceNumber(0u16), entry: &mut Number(42f64)};
        assert_eq!("Number", ne.name);
        assert_eq!(1u16, ne.id);
        assert_eq!(SequenceNumber(0u16), *ne.sequence);
        assert_eq!(42f64, match *ne.entry {
            Number(n) => n,
            _ => 0f64,
        });
        
        let se = Entry{name: "String", id: 2u16, sequence: &mut SequenceNumber(0u16), entry: &mut String("Test")};
        assert_eq!("String", se.name);
        assert_eq!(2u16, se.id);
        assert_eq!(SequenceNumber(0u16), *se.sequence);
        assert_eq!("Test", match *se.entry {
            String(s) => s,
            _ => "",
        });
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
