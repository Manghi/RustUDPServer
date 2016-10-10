#[macro_use]
extern crate mioco;
extern crate mio;
extern crate env_logger;
extern crate bincode;
extern crate rustc_serialize;
extern crate common;

use std::net;
use std::thread;
use std::time;
use std::io::{self, BufRead, Write};
use std::{str};

use common::communicate::*;
use common::packet::{Packet, MyLen, UDPData, UDPHeader};

/*
fn send_to_localhost_port(skt: &mio::udp::UdpSocket, ip: &net::Ipv4Addr, port: u16) {
    let send_addr1 = net::SocketAddrV4::new(*ip, port);
    let send_addr = net::SocketAddr::V4(send_addr1);

    let structmessage = Packet {
            header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
            data: UDPData { numerical: [1;10], textual: ['c','l','i','e','n','t',' ','h','i','i'], vector: vec![8675309, 10000, 2u32.pow(31)-1], other: vec![1;1392/4] },
        };

    println!("Message size: {} Bytes", structmessage.len());

    let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
    //udpHandler.socket.send_to(sentmsg_encoded.as_slice(), &net::SocketAddr::V4(target_addr));

    skt.send_to(sentmsg_encoded.as_slice(), &send_addr);
}
*/

fn print_help_menu() {
    println!("
Usage:
    help    - print this menu
    send    - send a message to the server
    exit    - quit the client

Example:
    > help

    > send

    > exit

");
}


fn send_to_localhost_port(skt: &mio::udp::UdpSocket, ip: &net::Ipv4Addr, port: u16) {
    let send_addr1 = net::SocketAddrV4::new(*ip, port);
    let send_addr = net::SocketAddr::V4(send_addr1);

    let structmessage = Packet {
            header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
            data: UDPData { numerical: [1;10], textual: ['c','l','i','e','n','t',' ','h','i','i'], vector: vec![8675309, 10000, 2u32.pow(31)-1], other: vec![1;1392/4] },
        };

    println!("Message size: {} Bytes", structmessage.len());

    let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();

    let _ = skt.send_to(sentmsg_encoded.as_slice(), &send_addr);
}

fn read_user_input(tx_user_input: &mioco::sync::mpsc::SyncSender<String>,
                   tx_exit_thread: &mioco::sync::mpsc::SyncSender<String>) {

    // temp, wait for other threads to instantiate
    let one_sec = time::Duration::from_millis(1000);
    thread::sleep(one_sec);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut line = String::new();

        stdin.lock().read_line(&mut line).ok().expect("Failed to read line");

        let line = line.parse::<String>().expect("Not a number");

        let mut command : Vec<&str> = line.trim().split(';').collect();
        //command[0].replace("\n","");

        for i in 0..command.len() {
            command[i] = command[i].trim();
        }

        //println!("Command: {:?}", command);

        match command[0].as_ref() {
            ""      => {},
            "help"  => {
                        println!("Help menu...");
                        print_help_menu();
                    },
            "send"  => {
                        println!("Sending to server");
                        let _ = tx_user_input.send(String::new());
                    },
            "exit" => {
                        let _ = tx_exit_thread.send(String::new());
                    },
            _      => {
                        println!("Command not recognized...");
                    },
        }

    }
}

fn start_transfer_socket(skt: &mio::udp::UdpSocket, rx_from_socket_chnl: &mioco::sync::mpsc::Receiver<std::string::String>) {
    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
/*    let mut buf = [0u8; 1024*16];

    loop {
        if let Some((len, addr)) = skt.recv_from(&mut buf).unwrap() {
            println!("Recieved!!!!!!!!!!!!!!!!");
        }
        let _m = rx_from_socket_chnl.try_recv().expect("No message");

        if !_m.is_empty() {
            send_to_localhost_port(&skt, &ip, 8890);// get_port_server());
        }
    }
    */
    loop {
        let _m = rx_from_socket_chnl.recv();//.expect("No message");
        send_to_localhost_port(&skt, &ip, 8890);// get_port_server());
    }
}

fn listen_on_socket(listen_addr: &net::SocketAddrV4) {
    let skt = socket(net::SocketAddr::V4(*listen_addr));
    let mut buf = [0u8; 1024 * 16];

    loop {
        if let Some((len, addr)) = skt.recv_from(&mut buf).unwrap() {
            println!("Length: {}, Addr: {}", len, addr);

            let data = Vec::from(&buf[0..len]);

            let decoded: Packet = bincode::rustc_serialize::decode(&data[..]).unwrap();

            println!("{:?}", decoded);
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let (tx_user_input, rx_user_input) = mioco::sync::mpsc::sync_channel::<String>(5);
    let (tx_to_socket, rx_from_socket_chnl) = mioco::sync::mpsc::sync_channel::<String>(5);
    let (tx_exit_thread, rx_exit_thread) = mioco::sync::mpsc::sync_channel::<String>(5);

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    let listen_addr = net::SocketAddrV4::new(ip, 8888);//get_port_client());
    let mut skt = socket(net::SocketAddr::V4(listen_addr));

    thread::spawn(move|| {
        read_user_input(&tx_user_input, &tx_exit_thread);
    });

    thread::spawn(move|| {
        start_transfer_socket(&skt, &rx_from_socket_chnl);
    });


    thread::spawn(move|| {
        listen_on_socket(&listen_addr);
    });

    mioco::start(move || {
                loop {
                    select!(
                        r:rx_user_input => {
                            let _m = rx_user_input.recv();
                            let _ = tx_to_socket.send(String::new());
                            //println!("1. Received ...");
                            //println!("{:?}", message.unwrap());
                        },
                        r: rx_exit_thread => {
                            let _m = rx_exit_thread.recv();
                            println!("Gracefully exiting...");
                            break;
                        },
                    );
                }
    }).unwrap();
}
