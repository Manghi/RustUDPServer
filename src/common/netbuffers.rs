// We need a buffer for a sliding window of 32 packets
// We need a buffer for acks for those packets
// We need a buffer for ????

use packet::Packet;

struct NetworkBuffer<'a> {
        tx_packets: &'a Vec<Packet>,
        rx_acks: &'a Vec<bool>,

}

struct NetStatistics {
    packets_sent: u64,
    packets_recv: u64,
    packets_dropped: u64,
}

// cant do this right now
static mut udp_buffer: NetworkBuffer<&'static> = NetworkBuffer {
    sent_packets: Vec<Packet> = Vec::new(),
    rx_acks: Vec<bool> = Vec::new(),
}
