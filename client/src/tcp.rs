pub mod syn {

    use std::net::Ipv4Addr;
    use pnet::util::MacAddr;
    use pnet_packet::ethernet::{EtherTypes, MutableEthernetPacket};
    use pnet_packet::ip::IpNextHeaderProtocols;
    use pnet_packet::ipv4::{Ipv4Flags, MutableIpv4Packet};
    use pnet_packet::tcp::{MutableTcpPacket, TcpFlags, TcpOption};
    pub struct PartialTCPPacketData<'a> {
        pub destination_ip: Ipv4Addr,
        pub interface_ip: Ipv4Addr,
        pub interface_mac: &'a MacAddr,
        pub interface_name: &'a String,
    }
    pub fn build_tcp_syn_packet(
        partial_packet: &PartialTCPPacketData,
        tmp_packet: &mut [u8], port: u32, gateway_mac: MacAddr
    ) {
        const ETHERNET_HEADER_LEN: usize = 14;
        const IPV4_HEADER_LEN: usize = 20;

        let mut eth_header = MutableEthernetPacket::new(
            &mut tmp_packet[..ETHERNET_HEADER_LEN]
        ).unwrap();
        /*
        maybe behind NAT
         */
        eth_header.set_destination(gateway_mac);
        eth_header.set_source(*partial_packet.interface_mac);
        eth_header.set_ethertype(EtherTypes::Ipv4);

        let mut ip_header = MutableIpv4Packet::new(
            &mut tmp_packet[ETHERNET_HEADER_LEN..(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)]
        ).unwrap();

        ip_header.set_header_length(69);
        ip_header.set_total_length(52);
        ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
        ip_header.set_source(partial_packet.interface_ip);
        ip_header.set_destination(partial_packet.destination_ip);
        ip_header.set_identification(rand::random::<u16>());
        ip_header.set_ttl(64);
        ip_header.set_version(4);
        ip_header.set_flags(Ipv4Flags::DontFragment);

        let checksum = pnet_packet::ipv4::checksum(&ip_header.to_immutable());
        ip_header.set_checksum(checksum);

        let mut tcp_header = MutableTcpPacket::new(
            &mut tmp_packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)..]
        ).unwrap();

        tcp_header.set_source(rand::random::<u16>());
        tcp_header.set_destination(port as u16);
        tcp_header.set_flags(TcpFlags::SYN);
        tcp_header.set_window(64240);
        tcp_header.set_data_offset(8);
        tcp_header.set_urgent_ptr(0);
        tcp_header.set_sequence(0);

        tcp_header.set_options(
            &[
                TcpOption::mss(1460),
                TcpOption::sack_perm(),
                TcpOption::nop(),
                TcpOption::nop(),
                TcpOption::wscale(7)
            ]
        );

        let checksum = pnet_packet::tcp::ipv4_checksum(
            &tcp_header.to_immutable(),
            &partial_packet.interface_ip,
            &partial_packet.destination_ip
        );

        tcp_header.set_checksum(checksum);

    }
}
