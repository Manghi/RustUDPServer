use std::thread;
use std::time;
use std::net;
use std::slice;
use std::mem;

mod nethelper;

#[repr(C, packed)]
pub struct UDPHeader {
    pub signature: [u8; 4],
}

#[repr(C, packed)]
pub struct UDPData {
    pub numerical: [u8; 10],
    pub textual: [char;10],
}

#[repr(C, packed)]
pub struct Packet {
    pub header: UDPHeader,
    pub data: UDPData,
}

pub fn main()
{
    println!("UDP");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = net::SocketAddrV4::new(ip, 8888);
    let send_addr = net::SocketAddrV4::new(ip, 8889);

    let future = nethelper::listen(net::SocketAddr::V4(listen_addr));
    //let message: Vec<u8> = vec![1;10];

    let structmessage = Packet {
        header: UDPHeader { signature: [82, 85, 83, 84] },
        data: UDPData { numerical: [1;10], textual: ['a','b','c','d','e','f','g','h','i','j'] },
    };

    // give the thread 3s to open the socket
    thread::sleep(time::Duration::from_millis(3000));

        unsafe {
        //let sentmessage1: Vec<u32> = std::mem::transmute(structmessage);
        //let sentmessage: Vec<u8> = std::mem::transmute(sentmessage1);

        let p: *const Packet = &structmessage;     // the same operator is used as with references
        let p: *const u8 = p as *const u8;  // convert between pointer types
        let sentmessage1: &[u8] =  slice::from_raw_parts(p, mem::size_of::<Packet>());
        let sentmessage = Vec::from(sentmessage1);

        nethelper::send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), sentmessage);
        }

    println!("Waiting");

    let received = future.join().unwrap();
    println!("Got {} bytes", received.len());
    println!("Data: {:?}", received);
    assert_eq!(54, received.len());
    assert_eq!(82, received[0]);
}
