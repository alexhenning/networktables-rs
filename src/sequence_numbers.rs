/// Sequence Numbers are a special type of number
/// Implements [rfc1982](http://tools.ietf.org/html/rfc1982)
// TODO: Document
#[deriving(Show, Clone)]
pub struct SequenceNumber(pub u16);
const SEQUENCE_NUMBER_DIVIDING_POINT: u16 = 32768u16;

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
    use super::{SequenceNumber, SEQUENCE_NUMBER_DIVIDING_POINT};
    use std::rand;
    
    #[test]
    fn sequence_number_equality() {
        // TODO: why not? for n in rand::task_rng().gen_iter::<u16>().take(100) {
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

    // TODO: Manually test edge cases
}
