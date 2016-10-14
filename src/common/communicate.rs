    extern crate mio;
    extern crate net2;
    //use std::thread;
    use std::net;
    use net2::UdpBuilder;

    pub const MAX_PACKET_SIZE: usize = 1472;

    enum Port {
        Client = 8888,
        Server = 8890,
    }

    pub fn socket(listen_on: net::SocketAddr) -> mio::udp::UdpSocket {
      //let attempt = net::UdpSocket::bind(listen_on);
      //let attempt = mio::udp::UdpSocket::bind(&listen_on);

      let udp; //= UdpBuilder::new_v4().unwrap();

      match UdpBuilder::new_v4() {
          Ok(new_udp) => {
              udp = new_udp
          },
          Err(_) => {
              panic!("Could not instantiate a UDP Builder.")
          },
      }

      let _ = udp.reuse_address(true);

      let sock = udp.bind(listen_on);

      let socket : mio::udp::UdpSocket;
      match sock {
        Ok(sock) => {
          let _ = sock.set_nonblocking(true);

           //socket = mio::udp::UdpSocket::from_socket(sock).unwrap();

          //let result = mio::udp::UdpSocket::from_socket(sock);
          match mio::udp::UdpSocket::from_socket(sock) {
              Ok(mio_socket) => {
                  info!("Bound socket to {}", listen_on);
                  socket = mio_socket;
              },
              Err(_) => {
                  panic!("Could not create socket.");
              }
          }
          //socket = mio::udp::UdpSocket::from(sock);

          //socket = sock;
        },
        Err(err) => panic!("Could not bind: {}", err)
      }
      socket
    }

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
