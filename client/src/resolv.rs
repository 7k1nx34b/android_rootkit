pub mod dns {
    use std::net::{Ipv4Addr, UdpSocket};

    pub fn a_record(domain: &str) -> Ipv4Addr {
        let socket = UdpSocket::bind(
            format!(
                "{}:{}", Ipv4Addr::UNSPECIFIED.to_string(), "0"
            )
        ).unwrap();
        socket.connect("8.8.8.8:53").unwrap();

        let mut domain_fragmented: Vec<&str> = domain.split(".").collect();
        let tld = domain_fragmented.get(domain_fragmented.len()-1).unwrap().clone();

        domain_fragmented.remove(domain_fragmented.len()-1); // remove TLD index
        let domain_src = domain_fragmented.concat();

        let mut v1: Vec<u8> = vec![
            0xaa, 0xaa, 0x01, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        v1.extend(vec![domain_src.len() as u8]);
        v1.extend(domain_src.as_bytes());
        v1.extend(vec![tld.len() as u8]);
        v1.extend(tld.as_bytes()) ;
        v1.extend(vec![0x00, 0x00, 0x01, 0x00, 0x01]);

        socket.send(&v1 as &[u8]).unwrap();

        let mut buf = [0; 4096];
        let n = socket.recv(&mut buf).unwrap();

        let a = buf[n-4];
        let b  = buf[n-3];
        let c= buf[n-2];
        let d = buf[n-1];

        let host = Ipv4Addr::new(a, b, c, d);
        host // TODO support subdomain query


    }
}