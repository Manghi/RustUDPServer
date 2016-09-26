extern crate bincode;
extern crate rustc_serialize;
extern crate common;

use std::thread;
use std::time;
use std::net;
// use std::mem;
// use std::fmt;

use common::communicate;
use common::packet::{Packet, MyLen, UDPData, UDPHeader};

pub fn main()
{
    println!("Server...");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let client_listen_addr = net::SocketAddrV4::new(ip, communicate::get_port_client_listen());
    let server_send_addr = net::SocketAddrV4::new(ip, communicate::get_port_server_transmit());
    let server_listen_addr = net::SocketAddrV4::new(ip, communicate::get_port_server_listen());

    loop
    {
        let future = communicate::listen(net::SocketAddr::V4(server_listen_addr));

        println!("Waiting");

        let rcvdmsg = future.join().unwrap();

        let decoded: Packet = bincode::rustc_serialize::decode(&rcvdmsg[..]).unwrap();

        println!("Got {} bytes", rcvdmsg.len());
        println!("{:?}", decoded);

        let structmessage = Packet {
            header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
            data: UDPData { numerical: [1;10], textual: ['s','e','r','v','e','r',' ','h','i','i'], vector: vec![8675309, 10000, 2^32-1] },
        };

        println!("Message size: {} Bytes", structmessage.len());

        // give the thread 3s to open the socket
        // thread::sleep(time::Duration::from_millis(3000));

        {
            let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
            communicate::send_message(net::SocketAddr::V4(server_send_addr), net::SocketAddr::V4(client_listen_addr), sentmsg_encoded);
            println!("Message sent!");
        }
    }
}
