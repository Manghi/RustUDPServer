#[macro_use]
extern crate mioco;
extern crate mio;
#[macro_use] extern crate log;
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
use common::packet::{Packet, MyLen};
use common::netbuffers::{ getNetworkBufferManager};

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

    let mut structmessage = Packet::new();
    structmessage.set_ack(4);
    structmessage.set_data(vec![1,2,3,4,5,6,7,8,9]);
    structmessage.inc_sequence_num();

    println!("Message size: {} Bytes", structmessage.len());

    match bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite) {
        Ok(msg) => {
            let encoded_packet : Vec<u8> = msg;
            let _ = skt.send_to(encoded_packet.as_slice(), &send_addr);
            println!("Sent");
        },
        Err(_) => {
            panic!("Could not encode packet!");
        }
    }
}

fn read_user_input(tx_user_input: &mioco::sync::mpsc::SyncSender<String>,
               tx_exit_thread: &mioco::sync::mpsc::SyncSender<String>) {

    // temp, wait for other threads to instantiate
    let one_sec = time::Duration::from_millis(1000);
    thread::sleep(one_sec);

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let stdin = io::stdin();
        let mut line = String::new();

        stdin.lock().read_line(&mut line).ok().expect("Failed to read line");

        let line = line.parse::<String>().expect("Not a number");

        let mut command : Vec<&str> = line.trim().split(';').collect();
        //command[0].replace("\n","");

        for i in 0..command.len() {
            command[i] = command[i].trim();
        }

        debug!("Command: {:?}", command);

        match command[0].as_ref() {
            ""      => {},
            "insert" => {
                println!("Inserting...");
                let _  = tx_user_input.send(String::from("insert"));
            },
            "remove" => {
                println!("Removing...");
                let _  = tx_user_input.send(String::from("remove"));
            },
            "query" => {
                match getNetworkBufferManager().lock() {
                    Ok(buffer) => {
                        println!("{:?}", *buffer);
                    },
                    Err(error) => println!("Unable to acquire lock: {}", error),
                }
            },
            "help"  => {
                println!("Help menu...");
                print_help_menu();
            },
            "send"  => {
                println!("Sending to server");
                let _ = tx_user_input.send(String::from("xfer"));
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

fn start_transfer_socket(skt: &mio::udp::UdpSocket,
        rx_from_socket_chnl: &mioco::sync::mpsc::Receiver<std::string::String>) {

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);

    loop {
        let _m = rx_from_socket_chnl.recv();

        match _m {
            Ok(message) => {
                if message == "xfer" {
                    send_to_localhost_port(&skt, &ip, get_port_server());
                }
                else if message == "insert" {
                    println!("Insert:O");
                    match getNetworkBufferManager().lock() {
                        Ok(mut buffer) => {
                            let mut pkt = Packet::new();
                            pkt.set_sequence_number(5);
                            pkt.set_client_id(String::from("Mang"));
                            buffer.insert(pkt);
                        },
                        Err(error) => {println!("This is poison: {:?}", error);},
                    }
                }
                else if message == "remove" {
                    match getNetworkBufferManager().lock() {
                        Ok(mut buffer) => {
                            let pktnum = buffer.remove(5);
                            match pktnum {
                                Ok(pkt) => println!("Removed: {:?}", pkt),
                                Err(err) => println!("No packet to remove... {:?}", err),
                            }
                        },
                        Err(error) => {println!("This is poison: {:?}", error);},
                    }
                }
            },
            Err(_) => {}
        }
    }
}

fn listen_on_socket(listen_addr: &net::SocketAddrV4) {
    let skt = socket(net::SocketAddr::V4(*listen_addr));
    let mut buf = [0u8; 1024 * 16];

    loop {
        match skt.recv_from(&mut buf) {
            Ok(Some((len, addr))) => {
                info!("Length: {}, Addr: {}", len, addr);

                let data = Vec::from(&buf[0..len]);

                let decoded: Packet = bincode::rustc_serialize::decode(&data[..]).unwrap();

                info!("{:?}", decoded);
            },
            Ok(None) => {},
            Err(_) => {
                debug!("Failed... No data to receive.");
            }
        }
    }
}

fn main() {
    match env_logger::init() {
        Ok(_) => {
            info!("Environment logger started...");
        }
        Err(_) => {
            debug!("Could not start logger... Abort.");
            return;
        }
    }

    let (tx_user_input, rx_user_input) = mioco::sync::mpsc::sync_channel::<String>(5);
    let (tx_to_socket, rx_from_socket_chnl) = mioco::sync::mpsc::sync_channel::<String>(5);
    let (tx_exit_thread, rx_exit_thread) = mioco::sync::mpsc::sync_channel::<String>(5);

    let ip = net::Ipv4Addr::new(0, 0, 0, 0);
    let listen_addr = net::SocketAddrV4::new(ip, 8888);//get_port_client());
    let skt = socket(net::SocketAddr::V4(listen_addr));

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
                    match rx_user_input.recv(){
                        Ok(message) => {
                            if message  == String::from("xfer") {
                                let _ = tx_to_socket.send(message);
                            }
                            else if message == String::from("insert") {
                                let _ = tx_to_socket.send(message);
                            }
                            else if message == String::from("remove") {
                                let _ = tx_to_socket.send(message);
                            }
                        },
                        Err(error) => println!("Error encountered! {}", error)
                    }
                },
                r: rx_exit_thread => {
                    let _m = rx_exit_thread.recv();
                    println!("Gracefully exiting...");
                    break;
                },
            );
        }
    }).unwrap(); // It's alright if this code panics.

    println!("Exiting client..");
}
