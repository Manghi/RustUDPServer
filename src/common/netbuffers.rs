/*
 * A NetworkBufferManager is used to keep track of what packets we have receieved thus far. It is not meant
 * to act as a buffer that is filled and then processed. Packets are processed immediately upon recepient.
 *
 * In this section I will use "sender" and "receiver" to indicate xfer between client and server.
 *
 * Normal case is that sender will send a stream of sequential packets. They will buffer each packet
 * only to discard when the receiver has confirmed the arrival of the packet. It will keep track of
 * which packets it has sent so far. This is used later on (see below). The receiver will then,
 * upon arrival of packet X, will flip the Xth ACK bit and process the packet. After processing,
 * they will reply to the sender with an ACK packet. This contains a bitmap of all ACKS receieved.
 *
 * When an ACK pkt is received by the Sender, XOR this value with its own knowledge of what it has sent.
 * The resulting set bits will determine which packets the sender sent by never were received by the
 * receiver. These now become candidate for high priority packets.
 *
 * Not all packets will be considered high priority. It depends on the message packet type and
 * its contents. You wouldn't want to resend a very stale, seconds-old packet.
 * So far, I'm thinking game initiation, completion, and collision attacks will need to be resent.
 * An option to maintain performance would be to interleve these resendable packets with the
 * standard stream. We can only have a window of 32 packets so there may be some throttling involved
 * when we do not wish to overwrite previous packets.
 *
 * Packets in groups of oldest 8 can be released after all have been received. The "elder" groups
 * must be released sequentially. They cannot jump around. The exception to this is if
 * there is a non-leading non-high priority missing packet, then we'll allow them to be released early.
 * After some time, these packets will become expired so as to prevent the sender from starvation.
 * Some of these metrics will be a tuneable parameter as it requires testing.
 *
 * Consider the following:
 *
 * Newest                       Oldest
 *      (4)      (3)      (2)      (1)
 * 0b01111111_11110111_10101111_11111111
 *                      H H
 *  Group 1 can be released early as all sent packets have been received.
 *  Group 3 cannot be released until group 2 has been released.
 *  Group 2 cannot release until its HP packets have been received by the receiver.
 *  Group 4 cannot be released because its leading packet has not been receieved. It may expire.
 *
 *
 * The ACK reply can be embedded in its own packets-to-send. A timer might be used to determine when
 * to send a periodic ACK if the receiver has no need for packets-to-send.
 *
 *
 * Sample flow:
 *          CLIENT                                                  SERVER
 *
 *          Send Packet_0                 ----->                Receive Packet_0
 *                                                              Process Packet_0
 *          Receive ACK (RAck=0b0001)     <-----                Send ACK
 *          XOR (RAck^Sent_Packets). No HP promotions.
 *          Send Packet_1                 ----->                N/A
 *          Send Packet_2                 ----->                N/A
 *          Send Packet_3                 ----->                Receive Packet_3
 *                                                              Process Packet_3
 *          Receive ACK (RAck=0b1001)     <-----                Send ACK
 *          XOR == (0b0110).
 *          Evaluate HP candiacy of Packet 1 & 2.
 *          HP = {Packet_1 and _2}
 *          Send Packet_1                 ----->                N/A
 *          Send Packet_2                 ----->                Receive Packet_2
 *                                                              Process Packet_2
 *          Receive ACK (RAck=0b1101)     <-----                Send ACK
 *          Evaluate HP candidacy. Packet 1 already HP.
 *          HP = {Packet_1}
 *          Send Packet_2                 ----->                Receive Packet_1
 *                                                              Process Packet_1
 *          Receive ACK (RAck=0b1111)     <-----                Send ACK
 *          All sent bits received.
 *          Process/Wait for more packets.
 */

use packet::Packet;

const MAX_PACKET_BUFFER_SIZE: usize = 32;

#[derive(PartialEq)]
enum NetworkBufferManagerProbe {
    Inserted,
    Exists,
    Full,
//    Empty,
}

struct NetworkBufferManager {
        sent_packet_buffer: Vec<Packet>,
        tx_packets: Vec<bool>,
        rx_acks: Vec<bool>,
        high_priority_acks:Vec<bool>,
        length: usize,
}

/*
struct NetStatistics {
    packets_sent: u64,
    packets_recv: u64,
    packets_dropped: u64,
}
*/

impl NetworkBufferManager {

    fn new() -> NetworkBufferManager {
        NetworkBufferManager {
            sent_packet_buffer: vec![Packet::new(); MAX_PACKET_BUFFER_SIZE],
            tx_packets: vec![false; MAX_PACKET_BUFFER_SIZE],
            rx_acks: vec![false; MAX_PACKET_BUFFER_SIZE],
            high_priority_acks: vec![false; MAX_PACKET_BUFFER_SIZE],
            length : 0,

        }
    }

    fn len(&self) -> usize {
        self.length
    }

    fn insert(&mut self, packet: &Packet) -> Result<NetworkBufferManagerProbe, NetworkBufferManagerProbe> {
        if self.is_full() {
            return Result::Err(NetworkBufferManagerProbe::Full)
        }
        else {
            let ack_indx = packet.get_sequence_num() % 32;
            let ack_num = ack_indx as usize;

            // If we've already received a packet with this ack# ignore it
            if !self.rx_acks[ack_num] {

                self.sent_packet_buffer[ack_num] = packet.clone();
                self.rx_acks[ack_num] = true;
                self.length += 1;

                return Result::Ok(NetworkBufferManagerProbe::Inserted)
            }
            else {
                return Result::Ok(NetworkBufferManagerProbe::Exists)
            }
        }
    }

    fn is_full(&self) -> bool {
        (self.len() >= MAX_PACKET_BUFFER_SIZE)
    }
}


// --------------------------
// |   NetworkBufferManager Tests  |
// --------------------------

#[cfg(test)]
mod test {
        use packet::Packet;
        use netbuffers::{NetworkBufferManager, MAX_PACKET_BUFFER_SIZE, NetworkBufferManagerProbe};
        use utils::*;

        #[test]
        fn test_network_buffer_creation() {
            let udp_buffer: NetworkBufferManager = NetworkBufferManager::new();

            assert_eq!(udp_buffer.sent_packet_buffer.len(), MAX_PACKET_BUFFER_SIZE);
            assert_eq!(udp_buffer.rx_acks.len(), MAX_PACKET_BUFFER_SIZE);

            assert_eq!(udp_buffer.sent_packet_buffer.capacity(), MAX_PACKET_BUFFER_SIZE);
            assert_eq!(udp_buffer.rx_acks.capacity(), MAX_PACKET_BUFFER_SIZE);
        }

        #[test]
        fn test_network_buffer_insertion() {
            let mut udp_buffer: NetworkBufferManager = NetworkBufferManager::new();
            let mut temp_packet: Packet = Packet::new();
            let user_name = String::from("network buffer tester");
            let seq_num :u32 = 1000;
            let bfr_index :usize = (seq_num as usize) % MAX_PACKET_BUFFER_SIZE;

            temp_packet.set_sequence_number(seq_num);
            temp_packet.set_client_id(user_name.clone());
            temp_packet.set_ackbit(3); // assume we have already received ack for pkt 3

            match udp_buffer.insert(&temp_packet) {
                Ok(n) => {

                    if n == NetworkBufferManagerProbe::Inserted {
                        println!("[test_network_buffer_insertion] Packet inserted successfully.");

                        let inserted_packet: &Packet = &udp_buffer.sent_packet_buffer[bfr_index];

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
            let mut udp_buffer: NetworkBufferManager = NetworkBufferManager::new();
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

                        if n == NetworkBufferManagerProbe::Inserted {
                            println!("Packet inserted successfully.");

                            let inserted_packet: &Packet = &udp_buffer.sent_packet_buffer[index];
                            //let ack_bits = inserted_packet.get_ackbits();

                            //assert_eq!(is_bit_set(ack_bits, ack_bit), true);
                            assert_eq!(inserted_packet.get_sequence_num(), seq_num as u32);
                        }
                        else if n == NetworkBufferManagerProbe::Exists {
                            println!("Packet already present in buffer.");
                        }
                    },
                    Err(_) => {

                        // In this case we will have wrapped on our buffer and have tried to insert
                        // packet#33 of seq_num=1032.

                        let last_inserted_packet : &Packet = &udp_buffer.sent_packet_buffer[index-1];

                        assert_eq!(x, 32);
                        assert_eq!(last_inserted_packet.get_sequence_num(), 1031);
                        println!("Network Buffer is full! This should never occur.");
                    },
                }
                seq_num += 1;
            }
        }

        fn test_network_buffer_received() {

        }

        fn test_network_buffer_send_receive_sequence_normal() {

        }

        fn test_network_buffer_send_receive_sequence_dropped_packets() {

        }

        fn test_network_buffer_send_receive_sequence_high_priority_packets() {

        }

}
