//! The module handling `lan-party-tools network`.

use reqwest;
use systemstat;
use systemstat::{IpAddr, Platform};

/// Count the number of `true` most significative bits present in a `u8` slice.
///
/// Used to print the subnet masks returned by [`systemstat`] in the short `/XX` notation.
///
/// ```
/// assert_eq!(count_octets(vec![255, 255, 255, 0]), 24);
/// assert_eq!(count_octets(vec![255, 128, 0, 0], 9);
/// ```
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

/// Format a [`IpAddr`] as if it was a subnet mask.
///
/// ```
/// let ipv4_addr = IpAddr::V4(systemstat::Ipv4Addr::new(255, 255, 255, 0));
/// assert_eq!(fmt_net_mask(&ipv4_addr), "/24");
///
/// let ipv6_addr = IpAddr::V6(systemstat::Ipv6Addr::new(0xFFFF, 0xFFFF, 0, 0, 0, 0, 0, 0));
/// assert_eq!(fmt_net_mask(&ipv6_addr), "/32");
/// ```
fn fmt_net_mask(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("/__"),
        IpAddr::Unsupported => format!("/??"),
        IpAddr::V4(ip) => format!("/{}", count_octets(ip.octets().as_ref())),
        IpAddr::V6(ip) => format!("/{}", count_octets(ip.octets().as_ref()))
    }
}

/// Format a [`IpAddr`] as if it was a regular IP address.
///
/// ```
/// let ipv4_addr = IpAddr::V4(systemstat::Ipv4Addr::new(192, 168, 1, 1));
/// assert_eq!(fmt_ip_addr(&ipv4_addr), "192.168.1.1");
/// ```
fn fmt_ip_addr(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("___"),
        IpAddr::Unsupported => format!("???"),
        IpAddr::V4(ip) => format!("{}", &ip),
        IpAddr::V6(ip) => format!("{}", &ip)
    }
}

/// Format a [`systemstat::NetworkAddrs`] as if it was a IP address and subnet mask pair.
///
/// ```
/// let addrs = systemstat::NetworkAddrs {
///    addr: IpAddr::V4(systemstat::Ipv4Addr::new(192, 168, 1, 1)),
///    netmask: IpAddr::V4(systemstat::Ipv4Addr::new(255, 255, 255, 0))
/// }
/// assert_eq!(fmt_net_addr(&addr), "192.168.1.1/24");
/// ```
fn fmt_net_addr(addrs: &systemstat::NetworkAddrs) -> String {
    let ip_string = fmt_ip_addr(&addrs.addr);
    let nm_string = fmt_net_mask(&addrs.netmask);
    return format!("{}{}", &ip_string, &nm_string);
}

/// Decide if a certain IP address should be displayed when `lan-party-tools network` is called.
///
/// Loopback and link local addresses return `false`, while all other addresses return `true`.
///
/// ```
/// assert_eq!(ip_addr_should_be_displayed(IpAddr::V4(systemstat::Ipv4Addr::new(192, 168, 1, 1))), true)
/// assert_eq!(ip_addr_should_be_displayed(IpAddr::V4(systemstat::Ipv4Addr::new(127, 0, 0, 1))), false)
/// ```
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

/// Decide if a certain network should be displayed when `lan-party-tools network` is called.
///
/// It calls [`ip_addr_should_be_displayed`] on all IP addresses of the network, and returns `true` if any of them should be displayed.
fn network_should_be_displayed(network: &systemstat::Network) -> bool {
    for network_addrs in &network.addrs {
        let ip_addr = &network_addrs.addr;
        if ip_addr_should_be_displayed(ip_addr) {
            return true;
        }
    }
    false
}

/// Syncronously fetches the public IPv4 address of the current network connection from [api.ipify.org](https://api.ipify.org/).
fn fetch_public_ipv4() -> String {
    reqwest::blocking::get("https://api.ipify.org/").expect("Public IP request failed").text().expect("Could not parse Public IP response")
}

/// Syncronously fetches the public IPv6 address of the current network connection from [api6.ipify.org](https://api6.ipify.org/).
fn fetch_public_ipv6() -> String {
    reqwest::blocking::get("https://api6.ipify.org/").expect("Public IP request failed").text().expect("Could not parse Public IP response")
}

/// The function that is run when `lan-party-tools network` is called.
pub fn network_command() -> Result<(), &'static str> {
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

    Ok(())
}
