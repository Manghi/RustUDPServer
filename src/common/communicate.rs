    extern crate mio;
    extern crate net2;

    use std::net;
    use net2::UdpBuilder;
    use packet::*;


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


    pub fn build_packet(  ) -> Packet {
        Packet {
               header: UDPHeader {
                   signature: ['L', 'I', 'F', 'E'],
                   crc32: 0,
                   client_id: 0,
                   sequence_number: 0,
                   action_type: PacketDataType::SYNC,
                   rsvd: [0;3],
                   ack_num: 0,
                   ack_bits: 0
               },
               data: UDPData {
                   raw_data: vec![0;MAX_PACKET_SIZE - get_packet_header_size()],
               },
           }
    }

#[cfg(test)]
mod test {

    use utils::hash;
    use communicate::*;

    #[test]
    // Send and listen to the same socket (listen_addr), from another socket (send_addr)
    fn test_build_packet() {
        let username = String::from("LifeUser1");
        let hashed_username: u64 = hash(&username.clone());
        let mut synchronize_pkt = build_packet();

        assert_eq!(0, synchronize_pkt.get_sequence_num());
        synchronize_pkt.inc_sequence_num();
        synchronize_pkt.inc_sequence_num();
        assert_eq!(2, synchronize_pkt.get_sequence_num());

        // Testing wrapped case
        for _ in 0..32 {
            synchronize_pkt.inc_sequence_num();
        }
        assert_eq!(2, synchronize_pkt.get_sequence_num());

        synchronize_pkt.set_client_id(username);
        assert_eq!(synchronize_pkt.get_client_id(), hashed_username);

        synchronize_pkt.set_ack(5);
        synchronize_pkt.set_ackbit(31);

        assert_eq!(5, synchronize_pkt.get_ack());
        assert_eq!(1, synchronize_pkt.is_ackbit_set(31));
        assert_eq!(0, synchronize_pkt.is_ackbit_set(5));

        let packet_data = vec![100, 3, 122, 255];
        synchronize_pkt.set_raw_data(packet_data.clone());
        assert_eq!(packet_data[2] , synchronize_pkt.get_data().raw_data[2]);

    }

}
