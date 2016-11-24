/*
 * This module was based on the GafferOnGames article titled
 * "Reliability & Flow Control". Special thanks to Glenn
 * for his detailed write-up on addressing these issues with UDP.
 * Though the link may change, you can find this article at
 * http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/
 *
 * This module was ported over from his C++ example.
 *
 */

use std::net;
use std::mem;
use std::cmp::Ordering;
use std::option;
use std::fmt;
use std::collections::{VecDeque};
use net2::UdpBuilder;
use mioco;
use mio;

#[derive(PartialEq)]
enum State {
    Disconnected,
    Listening,
    Connecting,
    ConnectFail,
    Connected
}

enum Mode {
    None,
    Client,
    Server
}

enum Port {
    Client = 8888,
    Server = 8890,
}

struct Socket {
    socket: mio::udp::UdpSocket,
    is_open: bool,
}

impl Socket {

    pub fn open(listen_on: net::SocketAddrV4) -> Socket {

        let udp;

        match UdpBuilder::new_v4() {
          Ok(new_udp) => {
              udp = new_udp
          },
          Err(_) => {
              panic!("Could not instantiate a UDP Builder.")
          },
        }

        let _ = udp.reuse_address(true);

        let sock = udp.bind(listen_on);

        let result_socket;

        match sock {
            Ok(sock) => {
              let _ = sock.set_nonblocking(true);

              match mio::udp::UdpSocket::from_socket(sock) {
                  Ok(mio_socket) => {
                      info!("Bound socket to {}", listen_on);
                      let new_socket = Socket {
                          socket : mio_socket,
                          is_open : true,
                      };
                      result_socket = new_socket
                  },
                  Err(_) => {
                      panic!("Could not create socket.");
                  }
              }
            },
            Err(err) => {
                panic!("Could not bind: {}", err);
            }
        }
        result_socket
    }

    pub fn receive(&self, data: &mut Vec<u8>) {
        let mut buf = [0u8; 1024 * 16];

        match self.socket.recv_from(&mut buf) {
            Ok(Some((len, addr))) => {
                info!("Length: {}, Addr: {}", len, addr);

                let buf_as_vec = Vec::from(&buf[0..len]);
                mem::replace::<(Vec<u8>)>(data, buf_as_vec);
            },
            Ok(None) => {},
            Err(_) => {
                debug!("No data to receive...");
            }
        }
    }

    pub fn close(&self) {
        drop(&self.socket);
    }

    pub fn send(&self, ip: &net::Ipv4Addr, port: u16, data : Vec<u8>) {
        let send_addr1 = net::SocketAddrV4::new(*ip, port);
        let send_addr = net::SocketAddr::V4(send_addr1);

        self.socket.send_to(data.as_slice(), &send_addr);
    }
}

#[derive(Clone)]
struct Address {
    address : net::Ipv4Addr,
    port : u16
}

impl Address {
    pub fn new(address : net::Ipv4Addr, port : u16) -> Address {
        Address {
            address : address,
            port : port,
        }
    }

    pub fn getAddress(&self) -> net::Ipv4Addr {
        self.address.clone()
    }

    pub fn getPort(&self) -> u16 {
        self.port.clone()
    }

    pub fn empty_address() -> net::Ipv4Addr {
        net::Ipv4Addr::new(0,0,0,0)
    }
}

impl PartialEq  for Address {

    fn eq(&self, other: &Address) -> bool {
        self.address == other.address && self.port == other.port
    }

    fn ne(&self, other: &Address) -> bool {
        self.address != other.address
    }
}

impl PartialOrd  for Address {

    fn partial_cmp(&self, other: &Address) -> Option<Ordering> {
        Some(self.address.cmp(&other.address))
    }

    fn lt(&self, other: &Address) -> bool {
        self.address < other.address
    }
    fn le(&self, other: &Address) -> bool {
        self.address <= other.address
    }
    fn gt(&self, other: &Address) -> bool {
        self.address > other.address
    }
    fn ge(&self, other: &Address) -> bool {
        self.address >= other.address
    }
}


struct Connection {
    protocol_id : usize,
    timeout : f32,
    running : bool,
    mode : Mode,
    state : State,
    socket : Socket,
    timeout_accumulator : f32,
    address : Address,  // Our destination
}

impl Connection {
    pub fn new(protocol_id : usize, timeout : f32, port : u16) -> Connection {

        let ip = net::Ipv4Addr::new(0, 0, 0, 0);
        let listen_addr = net::SocketAddrV4::new(ip, port);

        let mut new_connection = Connection {
            protocol_id : protocol_id,
            timeout : timeout,
            mode : Mode::None,
            running : false,
            state : State::Disconnected,
            timeout_accumulator : 0.0,
            address : Address::new(ip, port),
            socket : Socket::open(listen_addr),
        };

        new_connection.ClearData();
        new_connection
    }

    pub fn Start(&mut self) -> bool {
        assert_eq!(self.running, false);

        println!("Starting connection on port (self.address.port){}", self.address.port);

        self.running = true;
        self.OnStart();
        true
    }

    pub fn Stop(&mut self) {
        assert!(self.IsRunning(), true);

        println!("Stop connection...");

        let connected = self.IsConnected();
        self.ClearData();
        self.socket.close();
        self.running = false;

        if connected {
            self.OnDisconnect();
        }
        self.OnStop();
    }

    pub fn IsRunning(&self) -> bool {
        self.running
    }

    pub fn Listen(&mut self) {
        println!("Server listening for connections\n");
        let isConnected = self.IsConnected();
        self.ClearData();

        if isConnected {
            self.OnDisconnect()
        }
        self.mode = Mode::Server;
        self.state = State::Listening;
    }

    pub fn Connect(&mut self, dest_addr : &Address) {
        println!("Connecting to {}:{}", dest_addr.getAddress(), dest_addr.getPort());

        let isConnected = self.IsConnected();
        self.ClearData();

        if isConnected {
            self.OnDisconnect();
        }

        self.mode = Mode::Client;
        self.state = State::Connecting;
        self.address = (*dest_addr).clone();
    }

    pub fn IsConnecting(&self) -> bool {
        self.state == State::Connecting
    }

    pub fn ConnectFailed(&self) -> bool {
        self.state == State::ConnectFail
    }

    pub fn IsConnected(&self) -> bool {
        self.state == State::Connected
    }

    pub fn IsListening(&self) -> bool {
        self.state == State::Listening
    }

    pub fn GetMode(&self) -> &Mode {
        &self.mode
    }

    pub fn Update(&mut self, deltaTime: f32) {
        assert!(self.IsRunning(), true);

        self.timeout_accumulator += deltaTime;

        if self.timeout_accumulator > self.timeout {
            if self.IsConnecting() {
                println!("Connection Attempt Timed Out");
                self.ClearData();
                self.state = State::ConnectFail;
                self.OnDisconnect();
            }
            else if self.IsConnected() {
                println!("Connection Timed Out");
                self.ClearData();
                if self.state == State::Connecting {
                    self.state = State::ConnectFail;
                }
                self.OnDisconnect();
            }
        }
    }

    fn SendPacket(&self, data: &Vec<u8>, size: usize) -> bool {
        assert_eq!(self.IsRunning(), true);

        if self.address.getAddress() == Address::empty_address() {
            return false;
        }

        let mut packet : Vec<u8> = Vec::with_capacity(size+4);
        packet[0] = 'L' as u8;
        packet[1] = 'I' as u8;
        packet[2] = 'F' as u8;
        packet[3] = 'E' as u8;

        // TODO: Integrate with my current functional framework

        mem::replace::<(Vec<u8>)>(&mut packet, (data.clone()));

        self.socket.send(&self.address.getAddress(), self.address.getPort(), packet);
        true
    }

    fn ReceivePacket(&self, data: &Vec<u8>, size: usize) {
        assert!(self.IsRunning(), true);

    }

    fn ClearData(&mut self) {
        self.state = State::Disconnected;
        self.timeout_accumulator = 0.0;
        self.address = Address::new(Address::empty_address().clone(), 0);
    }

    fn OnStart(&mut self) {

    }

    fn OnStop(&mut self) {

    }

    fn OnConnect(&mut self) {

    }

    fn OnDisconnect(&mut self) {

    }
}



















#[derive(Clone, Debug)]
struct PacketData {
    sequence: u32,
    size: u32,
    time: f32,
}

impl fmt::Display for PacketData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PacketData:\n
        Sequence: {}\n
        Time: {}\n", self.sequence, self.time)
    }
}

impl PartialEq  for PacketData {

    fn eq(&self, other: &PacketData) -> bool {
        self.sequence == other.sequence && self.size == other.size && self.time == other.time
    }

    fn ne(&self, other: &PacketData) -> bool {
        self.sequence != other.sequence && self.size != other.size && self.time != other.time
    }
}

pub fn sequence_more_recent( s1: &u32, s2: &u32, max_sequence: &u32 ) -> bool
{
    ( s1 > s2 ) && ( s1 - s2 <= max_sequence/2 ) ||
    ( s2 > s1 ) && ( s2 - s1 >  max_sequence/2 )
}

const MIN_QUEUE_SIZE: usize = 0;
const MAX_QUEUE_SIZE: usize = 0x20;

struct PacketQueue {
    queue : VecDeque<PacketData>,
}

impl PacketQueue {
    pub fn new() -> PacketQueue {
        PacketQueue {
            queue : VecDeque::new(),
        }
    }

    pub fn exists(&self, sequence: u32) -> bool {
        let mut iterator = self.queue.iter();

        let mut found = false;

        loop {
            match iterator.next() {
                Some(nextPacketData) => {
                    if nextPacketData.sequence == sequence {
                        found = true;
                        break;
                    }
                },
                None => {
                    break;
                },
            }
        }
        found
    }

    pub fn insert_sorted(&mut self, packet_data: PacketData, max_sequence: u32)
    {
        if self.queue.is_empty() {
            self.push_back(packet_data)
        }
        else {

            match self.front() {
                Some(front) => {
                    if !sequence_more_recent(&packet_data.sequence, &front.sequence, &max_sequence ) {
                        self.push_front(packet_data.clone());
                        return;
                    }
                },
                None => {}
            }

            match self.back() {
                Some(last) => {
                    if sequence_more_recent(&packet_data.sequence, &last.sequence, &max_sequence ) {
                        self.push_back(packet_data.clone());
                        return;
                    }
                },
                None => {}
            }

            let insertion_index = self.find_sequence_insertion_point(packet_data.sequence, max_sequence);

            if insertion_index != 0xFF && self.is_index_valid(insertion_index) {
                // Check that we are not inserting a packet which is already present
                match self.get_packet(insertion_index) {
                    Some(packet_at_index) => {
                        assert!(packet_at_index.sequence != packet_data.sequence);
                    },
                    None => {
                        panic!("How did we find a packet at this index? {}", insertion_index);
                    }
                }

                self.queue.insert(insertion_index, packet_data.clone());
            }
        }
    }

    pub fn verify_sequencing(&self, max_sequence: u32) {
        let mut iterator = self.queue.iter();

        let mut previous = iterator.clone().last();
        let mut previousPacket = previous.unwrap();

        loop {
            match iterator.next() {
                Some(nextPacketData) => {
                    assert!(nextPacketData.sequence <= max_sequence);

                    if nextPacketData != previousPacket  {
                        assert!( sequence_more_recent(&(nextPacketData.sequence), &(previousPacket.sequence), &max_sequence) );
                        previousPacket = nextPacketData;
                    }
                },
                None => {
                    break;
                },
            }
        }
    }

    pub fn get_packet(&self, index: usize) -> option::Option<&PacketData> {
        self.queue.get(index)
    }

    fn push_back(&mut self, data: PacketData) {
        self.queue.push_back(data);
    }

    pub fn front(&self) -> Option<PacketData> {
        match self.queue.front() {
            Some(pd) => {
                Some(pd.clone())
            },
            None => {
                None
            }
        }
    }

    pub fn back(&self) -> Option<PacketData> {
        match self.queue.back() {
            Some(pd) => {
                Some(pd.clone())
            },
            None => {
                None
            }
        }
    }

    fn push_front(&mut self, data: PacketData) {
        self.queue.push_front(data);
    }


    pub fn is_index_valid(&self, index: usize) -> bool {
        index > MIN_QUEUE_SIZE && index < MAX_QUEUE_SIZE
    }

    pub fn find_sequence_insertion_point(&self, sequence_num: u32, max_sequence: u32) -> usize {
        let mut insertion_index = 0xFF;
        {
            let mut iterator = self.queue.iter().enumerate();

            loop {
                match iterator.next() {
                    Some((index, nextPacketData)) => {
                        println!("INS_IDX:nextPacketData.sequence={}, sequence_num={}", nextPacketData.sequence, sequence_num);

                        if sequence_more_recent(&nextPacketData.sequence, &sequence_num, &max_sequence) {
                            insertion_index = index;
                            break;
                        }
                    },
                    None => {
                        println!("End of list. Insertion Index: {}", insertion_index);
                        break;
                    },
                }
            }
        }
        insertion_index
    }

    pub fn find_index_for_sequence(&self, sequence_num: u32) -> usize {
        let mut packet_index = 0xFF;
        {
            let mut iterator = self.queue.iter().enumerate();

            loop {
                match iterator.next() {
                    Some((index, nextPacketData)) => {
                        println!("GET_IDX:nextPacketData.sequence={}, sequence_num={}", nextPacketData.sequence, sequence_num);

                        if nextPacketData.sequence == sequence_num {
                            packet_index = index;
                            break;
                        }
                    },
                    None => {
                        println!("End of list. Sequence Number Not Found: {}", sequence_num);
                        break;
                    },
                }
            }
        }
        packet_index
    }

}





#[cfg(test)]
mod test {

    use net;

    #[test]
    fn TestSequenceMoreRecent() {
        assert_eq!(true, net::sequence_more_recent(&4, &1, &10));
        assert_eq!(false, net::sequence_more_recent(&1, &4, &10));
    }

    #[test]
    fn TestPacketQueueExists() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };

        packet_queue.push_back(packet_data.clone());
        packet_data.sequence += 1;

        packet_queue.push_back(packet_data.clone());
        packet_data.sequence += 1;

        packet_queue.push_back(packet_data.clone());
        packet_data.sequence += 1;

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(102), true);
        assert_eq!(packet_queue.exists(99), false);

    }

    #[test]
    fn TestInsertQueueAtHead() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(102), true);
        assert_eq!(packet_queue.exists(99), false);

        println!("{:?}", packet_queue.queue);

        packet_data.sequence = 99;
        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(99), true);
        let index = packet_queue.find_index_for_sequence(99);

        println!("{:?}", packet_queue.queue);

        assert_eq!(index, 0);
    }

    #[test]
    fn TestInsertQueueAtTail() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(102), true);
        assert_eq!(packet_queue.exists(103), false);

        packet_data.sequence = 103;
        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(103), true);
        let index = packet_queue.find_index_for_sequence(103);
        assert_eq!(index, 3);
    }

    #[test]
    fn TestInsertQueueAtMiddle() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 2;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(103), true);
        assert_eq!(packet_queue.exists(102), false);

        packet_data.sequence = 102;
        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(102), true);
        let index = packet_queue.find_index_for_sequence(102);
        assert_eq!(index, 2);
    }

    #[test]
    fn TestSequenceNotPresent() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 2;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(103), true);
        assert_eq!(packet_queue.exists(102), false);

        packet_data.sequence = 108;

        assert_eq!(packet_queue.exists(108), false);
        let index = packet_queue.find_index_for_sequence(108);
        assert_eq!(index, 0xFF);
    }

    #[test]
    fn TestVerifySequencingPass() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 2;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(103), true);
        assert_eq!(packet_queue.exists(102), false);

        packet_queue.verify_sequencing(0xFFFF); // assertions within will fail if not sorted
    }

    #[test]
    #[should_panic]
    fn TestVerifySequencingFail() {
        let mut packet_queue = net::PacketQueue::new();
        let mut packet_data = net::PacketData {
                sequence: 100,
                size: 100,
                time: 4.17,
        };
        let max_sequence = 0xFFFF as u32;

        packet_queue.push_front(packet_data.clone());
        packet_data.sequence -= 1;

        packet_queue.push_front(packet_data.clone());
        packet_data.sequence += 0xFFFF;

        packet_queue.push_front(packet_data.clone());

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(99), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(102), false);

        println!("{:?}", packet_queue.queue);


        packet_queue.verify_sequencing(0xFFFF); // assertions within will fail if not sorted
    }
}
