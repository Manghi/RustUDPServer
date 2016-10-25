// We need a buffer for a sliding window of 32 packets
// We need a buffer for acks for those packets
// We need a buffer for ????

use packet::Packet;

const MAX_PACKET_BUFFER_SIZE: usize = 32;

#[derive(PartialEq)]
enum NetworkBufferProbe {
    Inserted,
    Exists,
    Full,
//    Empty,
}

struct NetworkBuffer {
        tx_packets: Vec<Packet>,
        rx_acks: Vec<bool>,
        length: usize,
}

/*
struct NetStatistics {
    packets_sent: u64,
    packets_recv: u64,
    packets_dropped: u64,
}
*/

impl NetworkBuffer {

    fn new() -> NetworkBuffer {
        NetworkBuffer {
            //tx_packets: Vec::with_capacity(MAX_PACKET_BUFFER_SIZE),
            //rx_acks: Vec::with_capacity(MAX_PACKET_BUFFER_SIZE),
            tx_packets: vec![Packet::new(); MAX_PACKET_BUFFER_SIZE],
            rx_acks: vec![false; MAX_PACKET_BUFFER_SIZE],
            length: 0,
        }
    }

    fn len(&self) -> usize {
        self.length
    }

    fn insert(&mut self, packet: &Packet) -> Result<NetworkBufferProbe, NetworkBufferProbe> {
        if self.is_full() {
            return Result::Err(NetworkBufferProbe::Full)
        }
        else {
            let ack_indx = packet.get_sequence_num() % 32;
            let ack_num = ack_indx as usize;

            // If we've already received a packet with this ack# ignore it
            if !self.rx_acks[ack_num] {

                self.tx_packets[ack_num] = packet.clone();
                self.rx_acks[ack_num] = true;
                self.length += 1;

                return Result::Ok(NetworkBufferProbe::Inserted)
            }
            else {
                return Result::Ok(NetworkBufferProbe::Exists)
            }
        }
    }

    fn is_full(&self) -> bool {
        (self.len() >= MAX_PACKET_BUFFER_SIZE)
    }
}


// --------------------------
// |   NetworkBuffer Tests  |
// --------------------------

#[cfg(test)]
mod test {
        use packet::Packet;
        use netbuffers::{NetworkBuffer, MAX_PACKET_BUFFER_SIZE, NetworkBufferProbe};
        use utils::*;

        #[test]
        fn test_network_buffer_creation() {
            let udp_buffer: NetworkBuffer = NetworkBuffer::new();

            assert_eq!(udp_buffer.tx_packets.len(), MAX_PACKET_BUFFER_SIZE);
            assert_eq!(udp_buffer.rx_acks.len(), MAX_PACKET_BUFFER_SIZE);

            assert_eq!(udp_buffer.tx_packets.capacity(), MAX_PACKET_BUFFER_SIZE);
            assert_eq!(udp_buffer.rx_acks.capacity(), MAX_PACKET_BUFFER_SIZE);
        }

        #[test]
        fn test_network_buffer_insertion() {
            let mut udp_buffer: NetworkBuffer = NetworkBuffer::new();
            let mut temp_packet: Packet = Packet::new();
            let user_name = String::from("network buffer tester");
            let seq_num :u32 = 1000;
            let bfr_index :usize = (seq_num as usize) % MAX_PACKET_BUFFER_SIZE;

            temp_packet.set_sequence_number(seq_num);
            temp_packet.set_client_id(user_name.clone());
            temp_packet.set_ackbit(3); // assume we have already received ack for pkt 3

            match udp_buffer.insert(&temp_packet) {
                Ok(n) => {

                    if n == NetworkBufferProbe::Inserted {
                        println!("[test_network_buffer_insertion] Packet inserted successfully.");

                        let inserted_packet: &Packet = &udp_buffer.tx_packets[bfr_index];

                        println!("{:?}", inserted_packet);

                        let ack_bits = inserted_packet.get_ackbits();

                        assert_eq!(is_bit_set(ack_bits, 8), false); // We did not get the ack for this yet
                        assert_eq!(inserted_packet.get_sequence_num(), 1000 as u32);
                        assert_eq!(is_bit_set(ack_bits, 3), true);
                        assert_eq!(is_bit_set(ack_bits, 0), false);
                    }
                    else {
                        println!("[test_network_buffer_insertion] Packet already present in buffer.");
                    }
                },
                Err(_) => panic!("[test_network_buffer_insertion] Network Buffer is full! This should never occur."),
            }

        }

        #[test]
        fn test_network_buffer_fill() {
            let mut udp_buffer: NetworkBuffer = NetworkBuffer::new();
            let mut seq_num: u32 = 1000;

            for x in 0..33 {
                let mut temp_packet: Packet = Packet::new();
                let user_name = String::from("network buffer tester");

                temp_packet.set_sequence_number(seq_num);
                //temp_packet.set_ack(ack_bit as u32);
                //temp_packet.set_ackbit(ack_bit);
                temp_packet.set_client_id(user_name.clone());

                let index: usize = (seq_num as usize) % 32;

                match udp_buffer.insert(&temp_packet) {

                    Ok(n) => {

                        if n == NetworkBufferProbe::Inserted {
                            println!("Packet inserted successfully.");

                            let inserted_packet: &Packet = &udp_buffer.tx_packets[index];
                            //let ack_bits = inserted_packet.get_ackbits();

                            //assert_eq!(is_bit_set(ack_bits, ack_bit), true);
                            assert_eq!(inserted_packet.get_sequence_num(), seq_num as u32);
                        }
                        else if n == NetworkBufferProbe::Exists {
                            println!("Packet already present in buffer.");
                        }
                    },
                    Err(_) => {

                        // In this case we will have wrapped on our buffer and have tried to insert
                        // packet#33 of seq_num=1032.

                        let last_inserted_packet : &Packet = &udp_buffer.tx_packets[index-1];

                        assert_eq!(x, 32);
                        assert_eq!(last_inserted_packet.get_sequence_num(), 1031);
                        println!("Network Buffer is full! This should never occur.");
                    },
                }
                seq_num += 1;
            }
        }

        fn test_network_buffer_peek() {

        }

        fn test_network_buffer_remove() {

        }

}
