    extern crate mio;
    extern crate net2;

    use std::net;
    use net2::UdpBuilder;
    //use packet::Packet;


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

/*
#[cfg(test)]
mod test {

}
*/
