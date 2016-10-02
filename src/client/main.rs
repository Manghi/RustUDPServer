extern crate bincode;
extern crate rustc_serialize;
extern crate common;

use std::{net, time};


use common::communicate;
use common::packet::{Packet, MyLen, UDPData, UDPHeader};

pub fn main()
{
    println!("Client");

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    //let ip2 = net::Ipv4Addr::new(192,168,1,237);
    let ip2 = net::Ipv4Addr::new(127, 0, 0, 1);

    let client_listen_addr = net::SocketAddrV4::new(ip, communicate::get_port_client_listen());
    let client_send_addr = net::SocketAddrV4::new(ip, communicate::get_port_client_transmit());

    let target_addr = net::SocketAddrV4::new(ip2, communicate::get_port_server_listen());

    // Client will wait for reply on this socket
    let future = communicate::listen(net::SocketAddr::V4(client_send_addr));


    let structmessage = Packet {
        header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
        data: UDPData { numerical: [1;10], textual: ['c','l','i','e','n','t',' ','h','i','i'], vector: vec![8675309, 10000, 2u32.pow(31)-1], other: vec![1;1392/4] },
    };

    println!("Message size: {} Bytes", structmessage.len());

    {
        let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
        communicate::send_message(net::SocketAddr::V4(client_send_addr), net::SocketAddr::V4(target_addr), sentmsg_encoded);
    }
    //let one_sec = time::Duration::from_millis(1000);
    //std::thread::sleep(one_sec);

    println!("Waiting");

    let rcvdmsg = future.join().unwrap();

    let decoded: Packet = bincode::rustc_serialize::decode(&rcvdmsg[..]).unwrap();


    println!("Got {} bytes", rcvdmsg.len());
    println!("{:?}", decoded);

}
