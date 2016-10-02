extern crate bincode;
extern crate rustc_serialize;
extern crate common;

use std::{net, time};
use common::packet::{Packet, MyLen, UDPData, UDPHeader};

pub mod communicate;

pub struct ClientSocket {
        socket: net::UdpSocket,
        target: net::SocketAddrV4,
}

impl ClientSocket {
    fn new(local_addr: net::SocketAddrV4, target_addr: net::SocketAddrV4) -> ClientSocket {
        let local_skt = communicate::socket(local_addr);
         ClientSocket {
             socket: local_skt,
             target: target_addr,
         }
    }
}

pub fn main()
{
    println!("Client");

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    //let ip2 = net::Ipv4Addr::new(192,168,1,237);
    let ip2 = net::Ipv4Addr::new(127, 0, 0, 1);

    let client_addr = net::SocketAddrV4::new(ip, communicate::get_port_client());
    let target_addr = net::SocketAddrV4::new(ip2, communicate::get_port_server());

    let client = ClientSocket::new(client_addr, target_addr);

    // Client will wait for reply on this socket
    let future = communicate::listen(&client.socket);


    let structmessage = Packet {
        header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
        data: UDPData { numerical: [1;10], textual: ['c','l','i','e','n','t',' ','h','i','i'], vector: vec![8675309, 10000, 2u32.pow(31)-1], other: vec![1;1392/4] },
    };

    println!("Message size: {} Bytes", structmessage.len());

    {
        let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
        communicate::send_message(&client.socket, client.target, sentmsg_encoded);
    }
    //let one_sec = time::Duration::from_millis(1000);
    //std::thread::sleep(one_sec);

    println!("Waiting");

    let rcvdmsg = future.join().unwrap();

    let decoded: Packet = bincode::rustc_serialize::decode(&rcvdmsg[..]).unwrap();


    println!("Got {} bytes", rcvdmsg.len());
    println!("{:?}", decoded);

}
