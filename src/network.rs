#![feature(ip)]

use reqwest;
use systemstat;
use systemstat::{IpAddr, Platform};


fn count_octets(octets: &[u8]) -> u8 {
    let mut count: u8 = 0;
    for octet in octets {
        let mut current_bit: u8 = 0b1000_0000;
        while current_bit > 0 {
            if (octet & current_bit) == 0 {
                break;
            }
            count += 1;
            current_bit = current_bit >> 1;
        }
        if current_bit != 0 {
            break;
        }
    }
    count
}

fn fmt_net_mask(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("/__"),
        IpAddr::Unsupported => format!("/??"),
        IpAddr::V4(ip) => format!("/{}", count_octets(ip.octets().as_ref())),
        IpAddr::V6(ip) => format!("/{}", count_octets(ip.octets().as_ref()))
    }
}

fn fmt_ip_addr(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("___"),
        IpAddr::Unsupported => format!("???"),
        IpAddr::V4(ip) => format!("{}", &ip),
        IpAddr::V6(ip) => format!("{}", &ip)
    }
}

fn fmt_net_addr(addrs: &systemstat::NetworkAddrs) -> String {
    let ip_string = fmt_ip_addr(&addrs.addr);
    let nm_string = fmt_net_mask(&addrs.netmask);
    return format!("{}{}", &ip_string, &nm_string);
}

fn ip_addr_should_be_displayed(ip_addr: &IpAddr) -> bool {
    match ip_addr {
        IpAddr::Empty => {return false},
        IpAddr::Unsupported => {return false},
        IpAddr::V4(ipv4) => {
            if ipv4.is_loopback() {
                return false;
            }
            if ipv4.is_link_local() {
                return false;
            }
        },
        IpAddr::V6(ipv6) => {
            if ipv6.is_unicast_link_local_strict() {
                return false;
            }
        },
    }
    true
}

fn network_should_be_displayed(network: &systemstat::Network) -> bool {
    for network_addrs in &network.addrs {
        let ip_addr = &network_addrs.addr;
        if ip_addr_should_be_displayed(ip_addr) {
            return true;
        }
    }
    false
}

fn fetch_public_ipv4() -> String {
    reqwest::blocking::get("https://api.ipify.org/").expect("Public IP request failed").text().expect("Could not parse Public IP response")
}

fn fetch_public_ipv6() -> String {
    reqwest::blocking::get("https://api6.ipify.org/").expect("Public IP request failed").text().expect("Could not parse Public IP response")
}

pub fn network_command() {
    let sys = systemstat::System::new();
    let networks = sys.networks().expect("Could not get networks.");

    for network_tuple in networks {
        let network = network_tuple.1;

        if network_should_be_displayed(&network) {
            println!("{}", &network.name);

            for network_addrs in network.addrs {
                println!("{}", fmt_net_addr(&network_addrs))
            }

            print!("\n");
        }
    }

    println!("Public IP");
    let public_ipv4 = &fetch_public_ipv4();
    let public_ipv6 = &fetch_public_ipv6();
    println!("{}/32", &public_ipv4);
    if public_ipv6 != public_ipv4 {
        println!("{}/32", &public_ipv6);
    }
}
