// Uses bincode, rustc_serialize
// See lib.rs within this folder

use std::*;
use utils::*;
use crc::{crc32};
use debug::{is_debug_print_enabled};

/*
enum Actor {
        SERVER,
        CLIENT,
}
*/

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq, Clone)]
pub struct UDPHeader {
    pub signature: u32,
    pub crc32: u32,
    pub client_id: u64,         // hash of username?
    pub sequence_number: u32,
    pub ack_num: u32,
    pub ack_bits: u32,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq, Clone)]
pub struct UDPData {
    pub raw_data : Vec<u8>,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq, Clone)]
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
        write!(f, "Signature: {:?}
  Sequence_Number:\t{:?}
  CRC32:\t{:?}
  Client_ID:\t{:?}
  Ack_Num:\t{:?}
  Ack_Bits:\t{:?}",
         self.signature,
         self.sequence_number,
         self.crc32,
         self.client_id,
         self.ack_num,
         self.ack_bits)
    }
}

impl fmt::Display for UDPData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "Raw Data:{:?}\n", "self.raw_data")
        write!(f, "Raw Data: {:?}\n", "ENABLE_RAW_DATA_PRINT_FOR_DETAIL")
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let debug_print = is_debug_print_enabled();
        if debug_print {
            write!(f, "\nPacket:\n  {:?}\n  {:?}\n", self.header, self.data)
        }
        else {
            write!(f, "Packet Debug Disabled...")
        }
    }
}

/*
impl fmt::Display for PacketDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for PacketDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
*/

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

    pub fn new() -> Packet {
        Packet {
               header: UDPHeader {
                   signature: 0x4C494645,
                   crc32: 0,
                   client_id: 0,
                   sequence_number: 0,
                   ack_num: 0,
                   ack_bits: 0
               },
               data: UDPData {
                   raw_data: vec![0;MAX_PACKET_SIZE - get_packet_header_size()],
               },
           }
    }

    pub fn set_signature(&mut self, signature: u32) {
        self.header.signature = signature;
    }

    pub fn get_signature(&self) -> u32 {
        self.header.signature
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
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

    pub fn set_ackbit(&mut self, bit:u32) {
        bit_set(&mut self.header.ack_bits, bit)
    }

    pub fn get_ackbits(&self) -> u32 {
        self.header.ack_bits
    }

    pub fn is_ackbit_set(&self, bit:u32) -> u32 {
        (self.header.ack_bits>>bit) & 1
    }

    pub fn set_client_id(&mut self, client_name: String) {
        self.header.client_id = hash(&client_name);
    }

    pub fn get_client_id(&self) -> u64 {
        self.header.client_id
    }

    pub fn set_sequence_number(&mut self, seq_num: u32) {
        self.header.sequence_number = seq_num
    }

    pub fn get_data(&self) -> &UDPData {
        &self.data
    }

    pub fn calculate_checksum(&mut self) {
        let checksum = crc32::checksum_ieee(&self.data.raw_data);
        println!("Checksum == {}", checksum);
        self.header.crc32 = checksum;
    }

    pub fn get_checksum(&self) -> u32 {
        self.header.crc32
    }
}

pub fn get_packet_header_size() -> usize {
    mem::size_of::<UDPHeader>()
}

#[cfg(test)]
mod test {

    use utils::hash;
    use packet::Packet;

    #[test]
    // Send and listen to the same socket (listen_addr), from another socket (send_addr)
    fn test_build_packet() {
        let username = String::from("LifeUser1");
        let hashed_username: u64 = hash(&username.clone());

        println!("Hash: {}", hashed_username);

        let mut synchronize_pkt = Packet::new();


        assert_eq!(0, synchronize_pkt.get_sequence_num());
        synchronize_pkt.inc_sequence_num();
        synchronize_pkt.inc_sequence_num();
        assert_eq!(2, synchronize_pkt.get_sequence_num());

        // Testing wrapped case
        for _ in 0..32 {
            synchronize_pkt.inc_sequence_num();
        }
        assert_eq!(2, synchronize_pkt.get_sequence_num());

        synchronize_pkt.set_client_id(username);
        assert_eq!(synchronize_pkt.get_client_id(), hashed_username);

        synchronize_pkt.set_ack(5);
        synchronize_pkt.set_ackbit(31);

        assert_eq!(5, synchronize_pkt.get_ack());
        assert_eq!(1, synchronize_pkt.is_ackbit_set(31));
        assert_eq!(0, synchronize_pkt.is_ackbit_set(5));

        let packet_data = vec![100, 3, 122, 255];
        synchronize_pkt.set_data(packet_data.clone());
        assert_eq!(packet_data[2] , synchronize_pkt.get_data().raw_data[2]);
    }

    #[test]
    fn test_packet_crc32_empty_packet() {
        let mut packet = Packet::new();

        packet.calculate_checksum();

        let checksum = packet.get_checksum();

        println!("{}", checksum);

        assert_eq!(checksum, 0x8EC868F8);
    }

    #[test]
    fn test_packet_crc32() {
        let username = String::from("LifeUser1");
        let mut packet = Packet::new();
        let packet_data = vec![100, 3, 122, 255];

        packet.set_data(packet_data.clone());
        packet.set_client_id(username);
        packet.calculate_checksum();

        let checksum = packet.get_checksum();

        assert_eq!(checksum, 0x6F947FE0);
    }
}
