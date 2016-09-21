<<<<<<< HEAD
extern crate bincode;
extern crate rustc_serialize;

use std::thread;
use std::time;
use std::net;
use std::mem;
use std::fmt;


mod nethelper;

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPHeader {
    pub signature: [char; 4],
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct UDPData {
    pub numerical: [u8; 10],
    pub textual: [char;10],
    pub vector: Vec<u32>,
}

#[repr(packed)]
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
=======
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
>>>>>>> 4041e08fcf914d64512d1e52b827f90bc05a2b78
pub struct Packet {
    pub header: UDPHeader,
    pub data: UDPData,
}

<<<<<<< HEAD
impl fmt::Debug for UDPHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) 
    }
}

impl fmt::Debug for UDPData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) 
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) 
    }
}

impl fmt::Display for UDPHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Signature:{:?}", self.signature)
    }
}

impl fmt::Display for UDPData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Numerical:{:?}\n  Textual:{:?}\n  Vector:{:?}", self.numerical, self.textual, self.vector)
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nPacket:\n  {:?}\n  {:?}\n", self.header, self.data)
    }
}

trait MyLen {
    fn len(&self) -> usize;
}

impl MyLen for Packet {
    // We already have the number of iterations, so we can use it directly.
    fn len(&self) -> usize {
        mem::size_of::<Packet>()
    }
}


=======
>>>>>>> 4041e08fcf914d64512d1e52b827f90bc05a2b78
pub fn main()
{
    println!("UDP");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = net::SocketAddrV4::new(ip, 8888);
    let send_addr = net::SocketAddrV4::new(ip, 8889);

    let future = nethelper::listen(net::SocketAddr::V4(listen_addr));
    //let message: Vec<u8> = vec![1;10];

    let structmessage = Packet {
<<<<<<< HEAD
        header: UDPHeader { signature: ['L', 'I', 'F', 'E'] },
        data: UDPData { numerical: [1;10], textual: ['a','b','c','d','e','f','g','h','i','j'], vector: vec![8675309, 10000, 2^32-1] },
    };

    println!("Message size: {} Bytes", structmessage.len());

    // give the thread 3s to open the socket
    thread::sleep(time::Duration::from_millis(3000));

/*        unsafe {
=======
        header: UDPHeader { signature: [82, 85, 83, 84] },
        data: UDPData { numerical: [1;10], textual: ['a','b','c','d','e','f','g','h','i','j'] },
    };

    // give the thread 3s to open the socket
    thread::sleep(time::Duration::from_millis(3000));

        unsafe {
>>>>>>> 4041e08fcf914d64512d1e52b827f90bc05a2b78
        //let sentmessage1: Vec<u32> = std::mem::transmute(structmessage);
        //let sentmessage: Vec<u8> = std::mem::transmute(sentmessage1);

        let p: *const Packet = &structmessage;     // the same operator is used as with references
        let p: *const u8 = p as *const u8;  // convert between pointer types
        let sentmessage1: &[u8] =  slice::from_raw_parts(p, mem::size_of::<Packet>());
        let sentmessage = Vec::from(sentmessage1);
<<<<<<< HEAD
*/
        {
            let sentmsg_encoded: Vec<u8> = bincode::rustc_serialize::encode(&structmessage, bincode::SizeLimit::Infinite).unwrap();
            nethelper::send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), sentmsg_encoded);
=======

        nethelper::send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), sentmessage);
>>>>>>> 4041e08fcf914d64512d1e52b827f90bc05a2b78
        }

    println!("Waiting");

<<<<<<< HEAD
    let rcvdmsg = future.join().unwrap();
    
    let decoded: Packet = bincode::rustc_serialize::decode(&rcvdmsg[..]).unwrap();

    
    println!("Got {} bytes", rcvdmsg.len());
    println!("{:?}", decoded);
    assert_eq!(structmessage.len(), decoded.len());
    assert_eq!(decoded, structmessage);
=======
    let received = future.join().unwrap();
    println!("Got {} bytes", received.len());
    println!("Data: {:?}", received);
    assert_eq!(54, received.len());
    assert_eq!(82, received[0]);
>>>>>>> 4041e08fcf914d64512d1e52b827f90bc05a2b78
}
