extern crate bincode;
extern crate rustc_serialize;
extern crate common;
extern crate mio;

//use std::io;
use std::net;
use mio::*;
use mio::deprecated::{EventLoop, Handler};
use mio::udp::*;

use common::communicate::*;
use common::packet::{Packet};//, MyLen, UDPData, UDPHeader};

pub const TOKEN_SERVER: Token = Token(10_000_000);

pub struct UdpHandler {
    socket: UdpSocket,
    packet_counter: u32,
}

impl UdpHandler {
    fn new(socket: UdpSocket) -> UdpHandler {
        UdpHandler {
            socket: socket,
            packet_counter: 0,
        }
    }
}


impl Handler for UdpHandler {
    type Timeout = ();
    type Message = u32;

    fn ready(&mut self, event_loop: &mut EventLoop<UdpHandler>, token: Token, events: Ready) {

           if events.is_readable() {
               match token {
                   TOKEN_SERVER => {
                       let mut buf: [u8; MAX_PACKET_SIZE] = [0; MAX_PACKET_SIZE];

                       let received = self.socket.recv_from(&mut buf);//.unwrap().unwrap();
                       println!("Received datagram...");

                       if let Some((size, sock)) = received.unwrap() {//.unwrap();

                            let addr = Some(sock);
                            //println!("bytes: {:?} from: {:?}", size, addr);

                            let data = Vec::from(&buf[0..size]);

                            let decoded: Packet = bincode::rustc_serialize::decode(&data[..]).unwrap();

                            self.packet_counter += 1;

                            let retaddr = addr.unwrap();
                            println!("{}", retaddr);


                            let one_sec = std::time::Duration::from_millis(1000);
                            std::thread::sleep(one_sec);

                            println!("{}", self.packet_counter);

                            // construct a reply
                            let _ = self.socket.send_to(&buf[0..size], &addr.unwrap());

                           //println!("We are receiving a datagram now...");
                           println!("Packet: {:?}", decoded);
                          // event_loop.shutdown();
                       }
                   },
                   _ => ()
               }
           }

           if events.is_writable() {
               println!("Event is writable...");
           }
       }

    fn notify(&mut self, event_loop: &mut EventLoop<UdpHandler>, msg: u32) {
        println!("Message notify received: {}", msg);
        event_loop.shutdown();
    }
}

/*
fn socket(listen_on: net::SocketAddr) -> mio::udp::UdpSocket {
  //let attempt = net::UdpSocket::bind(listen_on);
  let attempt = mio::udp::UdpSocket::bind(&listen_on);
  let socket;
  match attempt {
    Ok(sock) => {
      println!("Bound socket to {}", listen_on);
      socket = sock;
    },
    Err(err) => panic!("Could not bind: {}", err)
  }
  socket
}
*/

pub fn main()
{
    let mut event_loop = EventLoop::new().unwrap();

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    let listen_addr = net::SocketAddrV4::new(ip, get_port_server());
    let skt = socket(net::SocketAddr::V4(listen_addr));

    event_loop.register(&skt, TOKEN_SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    let _ = event_loop.run(&mut UdpHandler::new(skt));
}
