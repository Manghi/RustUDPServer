// Uses bincode, rustc_serialize
// See lib.rs within this folder

use std::*;
use utils::hash;

/*
enum Actor {
        SERVER,
        CLIENT,
}
*/

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub enum PacketDataType {
        SYNC,
        INSERTION,
        CAMERA,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPHeader {
    pub signature: [char; 4],
    pub crc32: u32,
    pub client_id: u64,         // hash of username?
    pub action_type: PacketDataType,
    pub rsvd: [u8;3],         // for word alignment
    pub sequence_number: u32,
    pub ack_num: u32,
    pub ack_bits: u32,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPData {
    pub raw_data : Vec<u8>,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Packet {
    pub header: UDPHeader,
    pub data: UDPData,
}


pub const MAX_PACKET_SIZE: usize = 1472;


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
        write!(f, "Raw Data:{:?}\n", self.raw_data)
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nPacket:\n  {:?}\n  {:?}\n", self.header, self.get_data())
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

impl Packet {
    pub fn set_raw_data(&mut self, data: Vec<u8>) {
        mem::replace::<(Vec<u8>)>(&mut self.data.raw_data, data);
    }

    pub fn get_sequence_num(&self) -> u32 {
        self.header.sequence_number
    }

    pub fn inc_sequence_num(&mut self) {
        let mut next_seq_num = self.get_sequence_num();
        next_seq_num += 1;
        next_seq_num %= u32::MAX.count_ones();
        self.header.sequence_number = next_seq_num;
    }

    pub fn set_ack(&mut self, ack: u32) {
        self.header.ack_num = ack;
    }

    pub fn get_ack(&self) -> u32 {
        self.header.ack_num
    }

    pub fn set_ackbit(&mut self, bit:u8) {
        self.header.ack_bits |= 1<<bit;
    }

    pub fn get_ackbits(&self) -> u32 {
        self.header.ack_bits
    }

    pub fn is_ackbit_set(&self, bit:u8) -> u32 {
        (self.header.ack_bits>>bit) & 1
    }

    pub fn set_client_id(&mut self, client_name: String) {
        self.header.client_id = hash(&client_name);
    }

    pub fn get_client_id(&self) -> u64 {
        self.header.client_id
    }

    pub fn get_data(&self) -> &UDPData {
        &self.data
    }
}

pub fn get_packet_header_size() -> usize {
    mem::size_of::<UDPHeader>()
}
