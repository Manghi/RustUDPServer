extern crate mioco;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate bincode;
extern crate rustc_serialize;
extern crate common;

use std::net::{SocketAddr, SocketAddrV4};
use std::io;
use mioco::udp::{UdpSocket};
use mioco::mio::Ipv4Addr;
use common::packet::*;
use common::communicate;
use common::net as mynet;

static mut packet_counter : u16 = 0;

fn listen_on_port(port: u16){
    mioco::spawn(move || -> io::Result<()> {
        let ip = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::V4(SocketAddrV4::new(ip, port));

        let mut sock;
        match UdpSocket::v4(){ // .unwrap();
            Ok(udpsocket) => {
                sock = udpsocket
            },
            Err(_) => {
                panic!("Could not create UdpSocket...");
            }
        }

        match sock.bind(&addr) {
            Ok(_) => {
                info!("Bound socket...");
            },
            Err(_) => {
                panic!("Could not bind on {}", addr);
            },
        }

        let mut buf = [0u8; 1024 * 16];

        loop {

            if let Some((len, addr)) = try!(sock.try_recv(&mut buf)) {
                info!("Length: {}, Addr: {}", len, addr);

                let data = Vec::from(&buf[0..len]);

                match bincode::rustc_serialize::decode::<Packet>(&data[..]) {
                    //  let decoded : Packet;
                    Ok(decoded) => {
                        info!("{:?}", decoded);
                        try!(sock.try_send(&mut buf[0..len], &addr));

                        // TODO: Please be safe!
                        unsafe {
                            packet_counter+=1;
                            info!("{}", packet_counter);
                        }

                    },
                    Err(_) => {
                        panic!("Could not decode message...");
                    },
                }
            }
        }

    });
}

fn main() {
    use std::time::Duration;
    use std::thread;

    let mut reliable_connection = mynet::ReliableConnection::new(0x4C494645, 6000000.0, 0xFFFFFFFF, mynet::Port::Server as u16);

    //reliable_connection.connection.address = net::Address::new( std::net::Ipv4Addr::new(127, 0, 0, 1) , net::Port::Client as u16);

    loop {
        let mut buffer = Vec::<u8>::with_capacity(100);
    //    for n in 0..100 {
//
//            buffer.insert(n, n as u8)
//        }

        let amount = reliable_connection.ReceivePacket(&mut buffer, 100*8);

        println!("Data received:\n{}\n{:?}\n\n", amount, buffer);

        thread::sleep(Duration::from_millis(1000));
    }
}
