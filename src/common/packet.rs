// Uses bincode, rustc_serialize
// See lib.rs within this folder

use std::mem;
use std::fmt;

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPHeader {
    pub signature: [char; 4],
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPData {
    pub numerical: [u8; 10],
    pub textual: [char;10],
    pub vector: Vec<u32>,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Packet {
    pub header: UDPHeader,
    pub data: UDPData,
}


impl fmt::Debug for UDPHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for UDPData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for UDPHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Signature:{:?}", self.signature)
    }
}

impl fmt::Display for UDPData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Numerical:{:?}\n  Textual:{:?}\n  Vector:{:?}", self.numerical, self.textual, self.vector)
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nPacket:\n  {:?}\n  {:?}\n", self.header, self.data)
    }
}

pub trait MyLen {
    fn len(&self) -> usize;
}

impl MyLen for Packet {
    // We already have the number of iterations, so we can use it directly.
    fn len(&self) -> usize {
        mem::size_of::<Packet>()
    }
}
