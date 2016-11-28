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
use packet as Packet;
use bincode;

#[derive(PartialEq)]
enum State {
    Disconnected,
    Listening,
    Connecting,
    ConnectFail,
    Connected
}

#[derive(PartialEq)]
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

    pub fn receive(&self, data: &mut Vec<u8>)  ->  Result<(usize, net::SocketAddr), &str> {
        let mut buf = [0u8; 1024 * 16];
        let mut received_bytes = 0;

        match self.socket.recv_from(&mut buf) {
            Ok(Some((len, addr))) => {
                info!("Length: {}, Addr: {}", len, addr);

                let buf_as_vec = Vec::from(&buf[0..len]);
                mem::replace::<(Vec<u8>)>(data, buf_as_vec);

                return Ok((len, addr));
            },
            Ok(None) => {
                return Err("Nothing to receive.");
            },
            Err(_) => {
                return Err("Nothing to receive.");
            }
        }
    }

    pub fn close(&self) {
        drop(&self.socket);
    }

    pub fn send(&self, ip: &net::Ipv4Addr, port: u16, data : Vec<u8>) -> bool {
        let send_addr1 = net::SocketAddrV4::new(*ip, port);
        let send_addr = net::SocketAddr::V4(send_addr1);

        match self.socket.send_to(data.as_slice(), &send_addr) {
            Ok(result) => {
                match result {
                    Some(bytes_written) => {
                        true
                    },
                    None => {
                        false
                    },
                }
            },
            Err(_) => {
                false
            },
        }
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
    protocol_id : u32,
    timeout : f32,
    running : bool,
    mode : Mode,
    state : State,
    socket : Socket,
    timeout_accumulator : f32,
    address : Address,  // Our destination

}

impl Connection {
    pub fn new(protocol_id : u32, timeout : f32, port : u16) -> Connection {

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

    pub fn Get_Protocol_Id(&self) -> u32 {
        self.protocol_id
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

        self.socket.send(&self.address.getAddress(), self.address.getPort(), data.clone())
    }

    fn ReceivePacket(&mut self, data: &mut Vec<u8>, size: usize) -> usize {
        assert!(self.IsRunning(), true);

        let mut recv_address : net::IpAddr = net::IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0));
        let mut recv_port : u16 = 0;
        let mut buffer = Vec::<u8>::new();
        let mut databfr = buffer.as_mut();
        let mut bytes_received = 0;

         match self.socket.receive(databfr) {
             Ok((amount, addr)) => {
                 bytes_received = amount;
                 recv_address = addr.ip();
                 recv_port = addr.port();
             },
             Err(_) => {
                 bytes_received = 0;
             },
         }

        if bytes_received <= 4 { // to do define this magic number
            return 0;
        }

        let decoded: Packet::Packet = bincode::rustc_serialize::decode(&databfr[..]).unwrap();

        if decoded.get_signature() != self.Get_Protocol_Id() || recv_port == 0 {
            return 0;
        }

        let socket_ip_address = Get_Ipv4Addr_From_IpAddr(recv_address);

        if (self.GetMode() == &Mode::Server) && !self.IsConnected() {
            println!("Server accepts from client {}:{}", recv_address, recv_port );
            self.state = State::Connected;

            self.address = Address::new(socket_ip_address, recv_port);
            self.OnConnect();

        }

        // Double check this condition.
        if self.address.getAddress() == socket_ip_address {
            if (self.mode == Mode::Client) && (self.state == State::Connecting) {
                println!("Client completed connection with server.");
                self.state == State::Connected;
            }

            self.timeout_accumulator = 0.0;

            let immutable_bfr = databfr.clone();
            mem::replace::<(Vec<u8>)>(data, immutable_bfr);

            return bytes_received;
        }

        return 0;
    }

    fn ClearData(&mut self) {
        self.state = State::Disconnected;
        self.timeout_accumulator = 0.0;
        self.address = Address::new(Address::empty_address().clone(), 0);
    }

    // TODO
    fn OnStart(&mut self) {

    }

    fn OnStop(&mut self) {
        self.ClearData();
    }

    fn OnConnect(&mut self) {

    }

    fn OnDisconnect(&mut self) {
        self.ClearData();
    }

    pub fn GetHeaderSize() -> usize {
        4
    }
}










struct ReliableSystem {
    max_sequence : u32,
    local_sequence : u32,
    remote_sequence : u32,

    sent_packets : u32,
    recv_packets : u32,
    lost_packets : u32,
    acked_packets: u32,

    sent_bandwidth : f32,
    acked_bandwidth : f32,
    rtt : f32,
    rtt_maximum : f32,

    acks : Vec<u32>,

    sentQueue : PacketQueue,
    pendingAckQueue : PacketQueue,
    receivedQueue : PacketQueue,
    ackedQueue : PacketQueue
}

impl ReliableSystem {
    pub fn new( max_sequence : u32) -> ReliableSystem {
        let mut reliability_system = ReliableSystem {
            max_sequence : max_sequence,
            local_sequence : 0,
            remote_sequence : 0,

            sent_packets : 0,
            recv_packets : 0,
            lost_packets : 0,
            acked_packets: 0,

            sent_bandwidth : 0.0,
            acked_bandwidth : 0.0,
            rtt : 0.0,
            rtt_maximum : 1.0,

            acks : Vec::<u32>::new(),

            sentQueue : PacketQueue::new(),
            pendingAckQueue : PacketQueue::new(),
            receivedQueue : PacketQueue::new(),
            ackedQueue : PacketQueue::new()
        };
        reliability_system.reset();

        reliability_system
    }

    pub fn reset(&mut self) {
        self.local_sequence = 0;
        self.remote_sequence = 0;
        self.sentQueue.clear();
        self.receivedQueue.clear();
        self.pendingAckQueue.clear();
        self.ackedQueue.clear();
        self.sent_packets = 0;
        self.recv_packets = 0;
        self.lost_packets = 0;
        self.acked_packets = 0;
        self.sent_bandwidth = 0.0;
        self.acked_bandwidth = 0.0;
        self.rtt = 0.0;
        self.rtt_maximum = 1.0;
    }

    pub fn PacketSent(&mut self, size: usize) {
        if self.sentQueue.exists(self.local_sequence) {
            println!("Local sequence {} exists in Sent Queue!", self.local_sequence);
        }

        assert!(self.sentQueue.exists(self.local_sequence), false);
        assert!(self.pendingAckQueue.exists(self.local_sequence), false);

        let mut data = PacketData {
            sequence : self.local_sequence,
            size : size as u32,
            time : 0.0,
        };

        self.sentQueue.push_back(data.clone());
        self.pendingAckQueue.push_back(data.clone());
        self.sent_packets += 1;
        self.local_sequence += 1;
        if self.local_sequence > self.max_sequence {
            self.local_sequence = 0;
        }
    }

    pub fn PacketReceived(&mut self, sequence: u32, size: usize) {
        self.recv_packets += 1;
        if self.receivedQueue.exists(sequence) {
            return
        }

        let mut data = PacketData {
            sequence : sequence,
            size : size as u32,
            time : 0.0,
        };

        self.receivedQueue.push_back(data.clone());
        if sequence_more_recent(&sequence, &self.remote_sequence, &self.max_sequence ) {
            self.remote_sequence = sequence;
        }
    }

    pub fn GenerateAckBits(&mut self) -> u32 {
        self.generate_ack_bits(self.get_remote_sequence(), &self.receivedQueue, self.max_sequence)
    }

    pub fn ProcessAck(&mut self, ack: u32, ack_bits: u32) {
        self.process_ack(ack, ack_bits);
    }

    pub fn Update(&mut self, deltaTime: f32) {
        self.acks.clear();
        self.AdvanceQueueTimes(deltaTime);
        self.UpdateQueues();
        self.UpdateStats();
        self.Validate();
    }

    pub fn Validate(&self) {
        let max_sequence = self.max_sequence;
        self.sentQueue.verify_sequencing(max_sequence);
        self.ackedQueue.verify_sequencing(max_sequence);
        self.pendingAckQueue.verify_sequencing(max_sequence);
        self.receivedQueue.verify_sequencing(max_sequence);
    }

    fn bit_index_for_sequence(&self, sequence: u32, ack: u32, max_sequence: u32) -> i32 {
        assert!(sequence != ack);
        assert!(sequence_more_recent(&sequence, &ack, &max_sequence) == false);

        if sequence > ack {
            assert!(ack < 33);
            assert!(max_sequence >= sequence);
            return (ack + (max_sequence - sequence)) as i32
        }
        else {
            assert!(ack >= 1);
            assert!(sequence <= ack - 1);
            return (ack - 1 - sequence) as i32
        }
    }

    fn generate_ack_bits(&self, ack: u32, receive_queue: &PacketQueue, max_sequence: u32) -> u32 {
        let mut ack_bits : u32 = 0;
        {
            let mut iterator = receive_queue.queue.iter();

            loop {
                match iterator.next() {
                    Some(next_packet) => {
                        if next_packet.sequence == ack || sequence_more_recent(&next_packet.sequence, &ack, &max_sequence) {
                            break;
                        }

                        let bit_index = self.bit_index_for_sequence(next_packet.sequence, ack, max_sequence);
                        if bit_index <= 31 {
                            ack_bits |= 1 << bit_index;
                        }
                    },
                    None => {break;},
                }
            }
        }
        ack_bits
    }

    fn process_ack(&mut self, ack: u32, ack_bits: u32) {

        if self.pendingAckQueue.queue.is_empty() {
            return;
        }

        let mut packet_index = 0xFF;
        {
            let mut iterator = self.pendingAckQueue.queue.iter();
            loop {
                match iterator.next() {
                    Some(packet_data) => {
                        let mut acked = false;

                        if packet_data.sequence == ack {
                            acked = true;
                        }
                        else if !sequence_more_recent(&packet_data.sequence, &ack, &self.max_sequence) {
                            let bit_index = self.bit_index_for_sequence(packet_data.sequence, ack, self.max_sequence);
                            if bit_index <= 31 {
                                acked = (1 & (ack_bits >> bit_index)) != 0;
                            }
                        }

                        if acked {
                            self.rtt += (packet_data.time - self.rtt) * 0.1;

                            self.ackedQueue.insert_sorted(packet_data.clone(), self.max_sequence);
                            self.acks.push(packet_data.sequence);
                            self.acked_packets += 1;

                            packet_index = self.pendingAckQueue.find_index_for_sequence(packet_data.sequence);
                        }
                    },
                    None => {break;},
                }
            }
        }

        if packet_index != 0xFF && self.pendingAckQueue.is_index_valid(packet_index) {
            self.pendingAckQueue.queue.remove(packet_index);
        }
    }

    pub fn get_local_sequence(&self) -> u32 {
        self.local_sequence
    }

    pub fn get_remote_sequence(&self) -> u32 {
        self.remote_sequence
    }

    pub fn get_max_sequence(&self) -> u32 {
        self.max_sequence
    }

    pub fn get_acks(&mut self, acks: &mut u32, count: &mut u32) {
        *acks = self.acks[0];
        *count = self.acks.len() as u32;
    }

    pub fn get_sent_packets(&self) -> u32 {
        self.sent_packets
    }

    pub fn get_received_packets(&self) -> u32 {
        self.recv_packets
    }

    pub fn get_lost_packets(&self) -> u32 {
        self.lost_packets
    }

    pub fn get_acked_packets(&self) -> u32 {
        self.acked_packets
    }

    pub fn get_sent_bandwidth(&self) -> f32 {
        self.sent_bandwidth
    }

    pub fn get_acked_bandwidth(&self) -> f32 {
        self.acked_bandwidth
    }

    pub fn get_round_trip_time(&self) -> f32 {
        self.rtt
    }

    pub fn GetHeaderSize() -> usize {
        12
    }

    pub fn AdvanceQueueTimes(&mut self, deltaTime: f32) {
        for packet in &mut self.sentQueue.queue {
            packet.time += deltaTime;
        }

        for packet in &mut self.receivedQueue.queue {
            packet.time += deltaTime;
        }

        for packet in &mut self.pendingAckQueue.queue {
            packet.time += deltaTime;
        }

        for packet in &mut self.ackedQueue.queue {
            packet.time += deltaTime;
        }
    }

    // TODO: The UpdateQueues method is not well thought out. My mostly-direct port looks pretty messy.
    // I'm thinking I can use a macro or something to accomplish it cleaner.
    // Need to investigate later.

    pub fn UpdateQueues(&mut self) {
        let epsilon : f32 = 0.0001;

        loop {
            match self.sentQueue.front() {
                Some(sent_packet) => {
                    if sent_packet.time > self.rtt_maximum + epsilon {
                        let _ = self.sentQueue.queue.pop_front();
                    }
                },
                None => {break;}
            }

            if self.sentQueue.queue.len() == 0 {
                break;
            }
        }

        if self.receivedQueue.queue.len() != 0 {
            match self.receivedQueue.back() {
                Some(received_packet) => {
                    let latest_sequence = received_packet.sequence;
                    let minimum_sequence = if latest_sequence >= 34  {latest_sequence - 34} else { self.max_sequence - (34 - latest_sequence)};

                    loop {
                        match self.receivedQueue.front() {
                            Some(recv_front_packet) => {
                                if !sequence_more_recent(&recv_front_packet.sequence, &minimum_sequence, &self.max_sequence) {
                                    let _ = self.receivedQueue.queue.pop_front();
                                }
                            },
                            None => {break;},
                        }

                        if self.receivedQueue.queue.len() == 0 {
                            break;
                        }
                    }
                },
                None => {},
            }
        }

        loop {
            match self.ackedQueue.front() {
                Some(acked_packet) => {
                    if acked_packet.time > (self.rtt_maximum*2.0) - epsilon {
                        let _ = self.ackedQueue.queue.pop_front();
                    }
                },
                None => {break;},
            }

            if self.ackedQueue.queue.len() == 0 {
                break;
            }
        }

        loop {
            match self.pendingAckQueue.front() {
                Some(pending_ack_packet) => {
                    if pending_ack_packet.time > self.rtt_maximum + epsilon {
                        let _ = self.pendingAckQueue.queue.pop_front();
                        self.lost_packets += 1;
                    }
                },
                None => {break;},
            }

            if self.pendingAckQueue.queue.len() == 0 {
                break;
            }
        }

    }

    pub fn UpdateStats(&mut self) {
        let mut sent_bytes_per_second: f32 = 0.0;

        for sent_packet in &self.sentQueue.queue {
            sent_bytes_per_second += sent_packet.size as f32;
        }

        let mut acked_packets_per_second = 0;
        let mut acked_bytes_per_second: f32 = 0.0;

        for acked_packet in &self.ackedQueue.queue {
            if acked_packet.time >= self.rtt_maximum {
                acked_packets_per_second += 1;
                acked_bytes_per_second += acked_packet.size as f32;
            }
        }

        sent_bytes_per_second /= self.rtt_maximum;
        acked_bytes_per_second /= self.rtt_maximum;

        self.sent_bandwidth = sent_bytes_per_second * (8.0/1000.0);
        self.acked_bandwidth = acked_bytes_per_second * (8.0/1000.0);
    }



}

















struct ReliableConnection {
    connection : Connection,
    reliability_system : ReliableSystem,
    packet_loss_mask : u32,
}

impl ReliableConnection {
    pub fn new(protocol_id: u32, timeout: f32, max_sequence : u32, port: u16) -> ReliableConnection {
        let mut reliableConnection = ReliableConnection {
            connection : Connection::new(protocol_id, timeout, port),
            reliability_system : ReliableSystem::new(max_sequence),
            packet_loss_mask : 0,
        };
        reliableConnection.connection.ClearData();
        reliableConnection
    }


    pub fn SendPacket(&mut self, data: Vec<u32>, size: usize) -> bool {
        let mut packet_to_send = Packet::Packet::new();

        packet_to_send.set_signature(self.connection.Get_Protocol_Id());
        packet_to_send.set_sequence_number(self.reliability_system.get_local_sequence());
        packet_to_send.set_ack(self.reliability_system.get_remote_sequence());
        packet_to_send.set_ackbit(self.reliability_system.GenerateAckBits());


        let encoded_packet : Vec<u8>;

        match bincode::rustc_serialize::encode(&packet_to_send, bincode::SizeLimit::Infinite) {
            Ok(msg) => {
                encoded_packet = msg;
            },
            Err(_) => {
                panic!("Could not encode packet!");
            }
        }

        let successful = self.connection.SendPacket(&encoded_packet, size);

        if !successful {
            return false;
        }

        self.reliability_system.PacketSent(size);
        true
    }

    pub fn ReceivePacket(&mut self, data: &mut Vec<u8>, size: usize) -> usize {
        let mut buffer = Vec::<u8>::new();
        let header_size = mem::size_of::<Packet::UDPHeader>();
        let received_bytes = self.connection.ReceivePacket(&mut buffer, size);

        let buffer_arr = buffer.as_slice();

        if received_bytes <= 12 {
            return 0;
        }

        let decoded_packet: Packet::Packet;

        // TODO : Move this into a 'Packet' interpreter from [u8] to Packet
        match bincode::rustc_serialize::decode(&buffer_arr[..]) {
            Ok(msg) => {
                decoded_packet = msg;
            },
            Err(_) => {
                panic!("Lets just panic for now...why could we not receieve a packet?");
            }
        }

        let data_bytes = received_bytes - header_size;

        self.reliability_system.PacketReceived(decoded_packet.get_sequence_num(), data_bytes);
        self.reliability_system.ProcessAck(decoded_packet.get_ack(), decoded_packet.get_ackbits());

        mem::replace::<(Vec<u8>)>(data, decoded_packet.get_data().raw_data.clone());
        data_bytes
    }

    pub fn Update(&mut self, deltaTime: f32) {
        self.connection.Update(deltaTime);
        self.reliability_system.Update(deltaTime);
    }


    pub fn GetHeaderSize(&self) -> u32 {
        (ReliableSystem::GetHeaderSize() as u32 + Connection::GetHeaderSize() as u32)
    }

    pub fn GetReliabilitySystem(&self) -> &ReliableSystem {
        &self.reliability_system
    }

    pub fn SetPacketLossMask(&mut self, mask: u32) {
        self.packet_loss_mask = mask;
    }

    fn ClearData(&mut self) {
        self.reliability_system.reset();
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

impl fmt::Display for PacketQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.queue)
    }
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

    pub fn clear(&mut self) {
        self.queue.clear();
    }

}





#[cfg(test)]
mod test {

    use net;
    use rand;

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
        let max_sequence = 0xFFFFFFFF  as u32;

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
        let max_sequence = 0xFFFFFFFF  as u32;

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
        let max_sequence = 0xFFFFFFFF  as u32;

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
        let max_sequence = 0xFFFFFFFF  as u32;

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
        let max_sequence = 0xFFFFFFFF  as u32;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 1;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);
        packet_data.sequence += 2;

        packet_queue.insert_sorted(packet_data.clone(), max_sequence);

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(103), true);
        assert_eq!(packet_queue.exists(102), false);

        packet_queue.verify_sequencing(0xFFFFFFFF ); // assertions within will fail if not sorted
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
        let max_sequence = 0xFFFFFFFF  as u32;

        packet_queue.push_front(packet_data.clone());
        packet_data.sequence -= 1;

        packet_queue.push_front(packet_data.clone());
        packet_data.sequence += 0xFFFFFFFF ;

        packet_queue.push_front(packet_data.clone());

        assert_eq!(packet_queue.exists(100), true);
        assert_eq!(packet_queue.exists(99), true);
        assert_eq!(packet_queue.exists(101), true);
        assert_eq!(packet_queue.exists(102), false);

        println!("{:?}", packet_queue.queue);


        packet_queue.verify_sequencing(0xFFFFFFFF ); // assertions within will fail if not sorted
    }

    #[test]
    fn TestPacketQueueStressTest()
    {
    	const MaximumSequence : u32 = 255;

    	let mut packet_queue = net::PacketQueue::new();

    	// check insert back
    	for i in  0..100
    	{
    		let mut packed_data = net::PacketData {
                    sequence: i,
                    size: 3,
                    time: 3.14
            };

    		packet_queue.insert_sorted( packed_data.clone(), MaximumSequence );
    		packet_queue.verify_sequencing( MaximumSequence );
    	}

    	// check insert front
    	packet_queue.clear();
    	for i in 0..100
    	{
            let mut packed_data = net::PacketData {
                    sequence: (100-i),
                    size: 3,
                    time: 3.14
            };

    		packet_queue.insert_sorted( packed_data.clone(), MaximumSequence );
    		packet_queue.verify_sequencing( MaximumSequence );
    	}

    	// check insert random
    	packet_queue.clear();
    	for i in 0..100
    	{
                let mut packed_data = net::PacketData {
                        sequence: rand::random::<u32>() % MaximumSequence,
                        size: 3,
                        time: 3.14
                };

    		    packet_queue.insert_sorted( packed_data.clone(), MaximumSequence );
    		    packet_queue.verify_sequencing( MaximumSequence );
    	}

        println!("{}", packet_queue);
        panic!("Yeah.");

    	// check wrap around
    	packet_queue.clear();
    	for i in 200..255
    	{
            let mut packed_data = net::PacketData {
                    sequence: i,
                    size: 3,
                    time: 3.14
            };


    		packet_queue.insert_sorted( packed_data.clone(), MaximumSequence );
    		packet_queue.verify_sequencing( MaximumSequence );
    	}
    	for i in 0..50
    	{
            let mut packed_data = net::PacketData {
                    sequence: i,
                    size: 3,
                    time: 3.14
            };

    		packet_queue.insert_sorted( packed_data.clone(), MaximumSequence );
    		packet_queue.verify_sequencing( MaximumSequence );
    	}
    }
}

fn Get_Ipv4Addr_From_IpAddr(ip_addr : net::IpAddr) -> net::Ipv4Addr {
    // TODO: this definitely can be written better
    let ipv4_str = format!("{}", ip_addr);
    let ipv4_vals : Vec<u8> = ipv4_str
        .split('.')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();

        assert_eq!(ipv4_vals.len(), 4);

    let sender_ipv4_addr = net::Ipv4Addr::new(ipv4_vals[0],ipv4_vals[1], ipv4_vals[2], ipv4_vals[3]);
    sender_ipv4_addr.clone()
}
