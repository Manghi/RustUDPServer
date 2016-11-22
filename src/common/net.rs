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
use std::collections::{VecDeque};
use net2::UdpBuilder;
use mioco;
use mio;

const NO_ADDRESS : &'static str = "0.0.0.0";

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


struct Address {
    address : String,
    port : u16
}

impl Address {
    pub fn new(address : String, port : u16) -> Address {
        Address {
            address : address,
            port : port,
        }
    }

    pub fn getAddress(&self) -> String {
        self.address.clone()
    }

    pub fn getPort(&self) -> u16 {
        self.port.clone()
    }

    // need to implement ==, !=, <, >
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
            address : Address::new(String::from("0.0.0.0"), 0),
            socket : Socket::open(listen_addr),
        };

        new_connection.ClearData();
        new_connection
    }

    pub fn Start(&mut self, port: u16) -> bool {
        assert_eq!(self.running, false);

        println!("Starting connection on port {}", port);

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

    pub fn Connect(&mut self) {
        println!("Connecting to {}", "_______");

        let isConnected = self.IsConnected();
        self.ClearData();

        if isConnected {
            self.OnDisconnect();
        }

        self.mode = Mode::Client;
        self.state = State::Connecting;
        //TODO: Figuruing out addressng
        //self.address = address;
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
        assert!(self.IsRunning(), true);

        if self.address.getAddress() == NO_ADDRESS {
            return false;
        }

        let mut packet : Vec<u8> = Vec::with_capacity(size+4);
        packet[0] = 'L' as u8;
        packet[1] = 'I' as u8;
        packet[2] = 'F' as u8;
        packet[3] = 'E' as u8;

        // TODO: Integrate with my current functional framework

        mem::replace::<(Vec<u8>)>(&mut packet, (data.clone()));
        true
    }

    fn ReceivePacket(&self, data: &Vec<u8>, size: usize) {
        assert!(self.IsRunning(), true);

    }

    fn ClearData(&mut self) {
        self.state = State::Disconnected;
        self.timeout_accumulator = 0.0;
        self.address = Address::new(String::from("0.0.0.0"), 0);
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



















#[derive(Clone)]
struct PacketData {
    sequence: u32,
    size: u32,
    time: f32,
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

            {
                let mut iterator = self.queue.iter().enumerate();

                loop {
                    match iterator.next() {
                        Some((index, nextPacketData)) => {
                            assert_eq!(nextPacketData.sequence, packet_data.sequence);
                            if sequence_more_recent(&nextPacketData.sequence, &packet_data.sequence, &max_sequence) {
                                self.queue.insert(index, packet_data.clone());
                                break;
                            }
                        },
                        None => {
                            println!("ERROR: Could not insert packet...");
                            break;
                        },
                    }
                }
            }
        }
    }

    pub fn push_back(&mut self, data: PacketData) {
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

    pub fn push_front(&mut self, data: PacketData) {
        self.queue.push_front(data);
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
}
