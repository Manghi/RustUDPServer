extern crate bincode;
extern crate rustc_serialize;
extern crate common;
extern crate mio;

use mio::*;
use mio::deprecated::{EventLoop, Handler};
use mio::udp::*;

use std::{net, time};
use common::packet::{Packet, MyLen, UDPData, UDPHeader};
use common::communicate::*;

pub const TOKEN_CLIENT: Token = Token(10_000_000);

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
                   TOKEN_CLIENT => {
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

//static mut testsocket : ClientSocket;

pub fn main()
{
    println!("Client");
    let mut event_loop = EventLoop::new().unwrap();

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    let listen_addr = net::SocketAddrV4::new(ip, get_port_client());
    let target_addr = net::SocketAddrV4::new(ip, get_port_server());
    let skt = socket(net::SocketAddr::V4(listen_addr));

    event_loop.register(&skt, TOKEN_CLIENT, Ready::readable(), PollOpt::edge()).unwrap();

    let ref mut udpHandler = UdpHandler::new(skt);

    let _ = event_loop.run(udpHandler);

    let structmessage = Packet {
            header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
            data: UDPData { numerical: [1;10], textual: ['c','l','i','e','n','t',' ','h','i','i'], vector: vec![8675309, 10000, 2u32.pow(31)-1], other: vec![1;1392/4] },
        };

        println!("Message size: {} Bytes", structmessage.len());

            let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
            udpHandler.socket.send_to(sentmsg_encoded.as_slice(), &net::SocketAddr::V4(target_addr));

}
