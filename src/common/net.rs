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

        // Socket receieve


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
