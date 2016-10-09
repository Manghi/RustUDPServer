extern crate mioco;
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

static mut packet_counter : u16 = 0;

fn listen_on_port(port: u16){
    mioco::spawn(move || -> io::Result<()> {
        let ip = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::V4(SocketAddrV4::new(ip, port));

        let mut sock = UdpSocket::v4().unwrap();

        sock.bind(&addr).unwrap();
        println!("Bound socket...");
        let mut buf = [0u8; 1024 * 16];
        loop {
            //let mut tmpbuf = [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9];
            //println!("Sending...");
            //try!(sock.send(&mut tmpbuf[0..10], &addr));
            if let Some((len, addr)) = try!(sock.try_recv(&mut buf)) {
                println!("Length: {}, Addr: {}", len, addr);
                //for i in 0..len {
                    //println!("Buffer: {:?}", buf[i]);


                    let data = Vec::from(&buf[0..len]);

                    let decoded: Packet = bincode::rustc_serialize::decode(&data[..]).unwrap();

                    unsafe {
                        packet_counter+=1;
                        println!("{}", packet_counter);
                    }

                    println!("{:?}", decoded);
                //}
                try!(sock.try_send(&mut buf[0..len], &addr));
            }
        }

    });
}

fn main() {
    env_logger::init().unwrap();

    mioco::start(move || {
        println!("Starting udp echo server on port: {}", communicate::get_port_server());

        //for port in START_PORT..START_PORT+1 {
            listen_on_port(communicate::get_port_server());
            //listen_on_port(START_PORT+1);
            println!("This is a test");
            //listen_on_port(START_PORT+2);
        //}
    }).unwrap();
}
