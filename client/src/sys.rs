pub mod system {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::net::Ipv4Addr;
    use std::process::{Command, exit};
    use pnet::util::MacAddr;

    pub fn get_device_imei() -> u64 {
        /*
            Result: Parcel(
                0x00000000: 00000000 0000000f 00350033 00320035 '........3.5.5.2.'
                0x00000010: 00360033 00350030 00360038 00350038 '3.6.0.5.8.6.8.5.'
                0x00000020: 00390038 00000034                   '8.9.4...        '
            )
            https://chromium.googlesource.com/infra/luci/python-adb/+/7b20f895c33a8a950a4e69b33c02108cb8c701f0/high_test.py
         */

        let imei = match Command::new("/system/bin/service") // RAW_IMEI
            .arg("call")
            .arg("iphonesubinfo")
            .arg("1")
            .output()
        {
            Ok(o) => {
                let mut friendly_imei = String::new();
                for line in String::from_utf8(o.stdout).unwrap().split("\n") {
                    if line.contains("\x27") {
                        let x27 = line.find("\x27").unwrap();
                        let len = line.len();
                        friendly_imei.push_str(
                            line[x27..len].to_string()
                                .replace("\x27", "")
                                .replace("\x2e", "")
                                .replace("\x29", "")
                                .as_str().trim()
                        );
                    }
                }
                friendly_imei // FRIENDLY_IMEI
            }
            Err(_) => { exit(0x0100); }
        };
        imei.parse().unwrap()
    }
    pub fn set_airplane() { // Re-receiving LTE signal
        for i in vec![1, 0] {
            Command::new("/sbin/su")
                .arg("-c")
                .arg("settings")
                .arg("put")
                .arg("global")
                .arg("airplane_mode_on")
                .arg(i.to_string())
                .output().unwrap();

            Command::new("/sbin/su")
                .arg("-c")
                .arg("am")
                .arg("broadcast")
                .arg("-a")
                .arg("android.intent.action.AIRPLANE_MODE")
                .output().unwrap();
        }
    }

    pub fn interface_is_connected_internet(interface_name: &str) -> bool {
        let res = match Command::new("/system/bin/ping")
            .arg("-I")
            .arg(interface_name)
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg("2")
            .arg("8.8.8.8")
            .output()
        {
            Ok(o) => {
                if String::from_utf8(o.stdout).unwrap().contains("icmp_seq") {
                    return true;
                }
                return false;
            },
            Err(_) => {return false;}
        };
        false;
    }
    pub fn read_internal_routing_table(target_interface_name: &str, suffix_octet: u8) -> Option<Ipv4Addr> {
        for line in BufReader::new(File::open("/proc/net/route").unwrap()).lines().skip(1) {
            let line = line.unwrap();
            let mut _fragmented: Vec<&str> = line.split_whitespace().collect();
            if _fragmented.get(0).unwrap().clone() == target_interface_name {
                let gateway_ip_octets = Ipv4Addr::from(
                    u32::from_be(u32::from_str_radix(_fragmented.get(1).unwrap().clone(), 16).unwrap())
                ).octets();
                return Some(
                    Ipv4Addr::new(
                        gateway_ip_octets[0],
                        gateway_ip_octets[1],
                        gateway_ip_octets[2],
                        gateway_ip_octets[3] + suffix_octet, // 1 or 254
                    )
                )
            }
        }
        return None
    }
    pub fn read_internal_arp_table(target_interface_name: &str, target_gateway_ip: Ipv4Addr) -> Option<MacAddr> {

        for line in BufReader::new(File::open("/proc/net/arp").unwrap()).lines().skip(1) {
            let line = line.unwrap();
            let mut _fragmented: Vec<&str> = line.split_whitespace().collect();

            if _fragmented.get(0).unwrap().clone() == target_gateway_ip.to_string().as_str() &&
                _fragmented.get(_fragmented.len() -1).unwrap().clone() == target_interface_name {

                let gateway_mac = _fragmented.get(3).unwrap().clone();
                let mut gateway_mac_fragmented = gateway_mac.split(":");

                let a: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();
                let b: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();
                let c: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();
                let d: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();
                let e: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();
                let f: u8 = u8::from_str_radix(gateway_mac_fragmented.nth(0).unwrap(), 16).unwrap();

                return Some(MacAddr(a,b,c,d,e,f))
            }
        }
        return None
    }

}