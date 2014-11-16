use std::error::Error;
use std::error::FromError;
use std::io::IoError;

pub type NtResult<T> = Result<T, NtError>;

#[deriving(PartialEq,Eq,Show,Clone)]
pub enum NtErrorKind {
    UnsupportedType(u8),
    StringConversionError,
    KeyAlreadyExists(String),
    IdAlreadyExists(u16),
    IdDoesntExist(u16),
    NetworkProblem(IoError),
}

#[deriving(PartialEq,Eq,Show,Clone)]
pub struct NtError {
    pub kind: NtErrorKind,
    // pub desc: &'static str,
    // pub detail: Option<String>,
    // pub cause: Option<Error>,
}

impl Error for NtError {
    fn description(&self) -> &str {
        match self.kind {
            UnsupportedType(_) => "Unsupported entry type.",
            StringConversionError => "Error parsing string.",
            KeyAlreadyExists(_) => "Key={} already exists.",
            IdAlreadyExists(_) => "ID={} already exists.",
            IdDoesntExist(_) => "ID={} Doesn't exists.",
            NetworkProblem(_) => "Problem connecting to server.",
        }
    }

    fn detail(&self) -> Option<String>{
        match self.kind {
            UnsupportedType(entry_type) => Some(format!("Unsupported entry type={}.", entry_type)),
            StringConversionError => None,
            KeyAlreadyExists(ref key) => Some(format!("Key={} already exists.", key)),
            IdAlreadyExists(id) => Some(format!("ID={} already exists.", id)),
            IdDoesntExist(id) => Some(format!("ID={} Doesn't exists.", id)),
            NetworkProblem(ref err) => err.detail(),
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
        NtError{kind: NetworkProblem(err)}
    }
}
