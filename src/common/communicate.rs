    extern crate mio;
    extern crate net2;
    //use std::thread;
    use std::net;
    use net2::UdpBuilder;
    //use mio::*;
    //use mio::udp::*;

    pub const MAX_PACKET_SIZE: usize = 1472;

    enum Port {
        Client = 8888,
        Server = 8890,
    }

    pub fn socket(listen_on: net::SocketAddr) -> mio::udp::UdpSocket {
      //let attempt = net::UdpSocket::bind(listen_on);
      //let attempt = mio::udp::UdpSocket::bind(&listen_on);

      let udp = UdpBuilder::new_v4().unwrap();
      udp.reuse_address(true);

      let sock = udp.bind(listen_on);

      let socket : mio::udp::UdpSocket;
      match sock {
        Ok(sock) => {
          println!("Bound socket to {}", listen_on);
          sock.set_nonblocking(true);

          socket = mio::udp::UdpSocket::from_socket(sock).unwrap();
          //socket = mio::udp::UdpSocket::from(sock);

          //socket = sock;
        },
        Err(err) => panic!("Could not bind: {}", err)
      }
      socket
    }

/*
    pub fn socket(listen_on: net::SocketAddrV4) -> net::UdpSocket {
      let attempt = net::UdpSocket::bind(listen_on);
      let socket;
      match attempt {
        Ok(sock) => {
          println!("Bound socket to {}", listen_on);
          socket = sock;
        },
        Err(err) => panic!("Could not bind: {}", err)
      }
      socket
    }


    fn read_message(socket: &net::UdpSocket) -> Vec<u8> {
      let mut buf: [u8; MAX_PACKET_SIZE] = [0; MAX_PACKET_SIZE];
      println!("Reading data");
      let result = socket.recv_from(&mut buf);
      //drop(socket);
      let data;
      match result {
        Ok((amt, src)) => {
          println!("Received data from {}", src);
          data = Vec::from(&buf[0..amt]);
        },
        Err(err) => panic!("Read error: {}", err)
      }
      data
    }

    pub fn send_message(socket: &net::UdpSocket, target: net::SocketAddrV4, data: Vec<u8>) {
      //let socket = socket(send_addr);
      println!("Sending data");
      let result = socket.send_to(&data, target);
      //drop(socket);
      match result {
        Ok(amt) => println!("Sent {} bytes", amt),
        Err(err) => panic!("Write error: {}", err)
      }
    }


    pub fn listen(listen_on: &net::UdpSocket) -> thread::JoinHandle<Vec<u8>> {
        //let socket = socket(listen_on);
        let handle = thread::spawn(move || {
            read_message(&listen_on);//.try_clone().as_ref().unwrap());
        });
        handle
    }*/

    pub fn get_port_client() -> u16 {
        Port::Client as u16
    }

    pub fn get_port_server() -> u16 {
        Port::Server as u16
    }


#[cfg(test)]
mod test {
  use std::net;
  use std::thread;
  use std::time;
  use super::*;

  #[test]
  // Send and listen to the same socket (listen_addr), from another socket (send_addr)
  fn test_udp() {
    println!("UDP");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = net::SocketAddrV4::new(ip, get_port_client_listen());
    let send_addr = net::SocketAddrV4::new(ip, get_port_server_listen());
    let future = listen(net::SocketAddr::V4(listen_addr));
    let message: Vec<u8> = vec![10];
 // give the thread 3s to open the socket
    thread::sleep(time::Duration::from_millis(3000));
    send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), message);
    println!("Waiting");
    let received = future.join().unwrap();
    println!("Got {} bytes", received.len());
    assert_eq!(1, received.len());
    assert_eq!(10, received[0]);
  }
}
