#![feature(ip)]
#![feature(str_strip)]

use clap;
use systemstat::{Platform, NetworkAddrs, IpAddr};
use reqwest;
use std::path::Path;
use std::fs;
use std::io;
use regex;
use lazy_static::lazy_static;
use std::fs::DirEntry;


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

fn fmt_ip_addr(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("___"),
        IpAddr::Unsupported => format!("???"),
        IpAddr::V4(ip) => format!("{}", &ip),
        IpAddr::V6(ip) => format!("{}", &ip)
    }
}

fn fmt_net_mask(ip: &IpAddr) -> String {
    match ip {
        IpAddr::Empty => format!("/__"),
        IpAddr::Unsupported => format!("/??"),
        IpAddr::V4(ip) => format!("/{}", count_octets(ip.octets().as_ref())),
        IpAddr::V6(ip) => format!("/{}", count_octets(ip.octets().as_ref()))
    }
}

fn fmt_net_addr(addrs: &NetworkAddrs) -> String {
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

fn get_default_steamapps() -> &'static Path {
    if cfg!(windows) {
        Path::new(r"C:\Program Files (x86)\Steam\steamapps")
    } else if cfg!(macos) {
        Path::new(r"~/Library/Application Support/Steam")
    } else if cfg!(linux) {
        Path::new(r"~/.steam/steam/steamapps")
    } else {
        unimplemented!("Unsupported platform!");
    }
}

fn find_manifests_in_folder(folder: &Path) -> Vec<DirEntry> {
    let mut manifests: Vec<DirEntry> = vec![];
    for object in fs::read_dir(folder).expect("Could not open manifests folder") {
        let object = object.expect("Could not get DirEntry for object");

        if !object.path().is_file() {
            continue;
        }

        if !object.file_name().to_str().unwrap().starts_with("appmanifest_") {
            continue;
        }

        manifests.push(object);
    };
    return manifests;
}

fn extract_appid_from_manifest_contents(string: &str) -> &str {
    lazy_static! {
        static ref NAME_REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"appid\"\\s+\"(.+)\"\\s*$").unwrap();
    }
    NAME_REGEX.captures(&string).expect("Could not find appid").get(1).expect("Could not find appid").as_str()
}

fn extract_game_name_from_manifest_contents(string: &str) -> &str {
    lazy_static! {
        static ref NAME_REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"name\"\\s+\"(.+)\"\\s*$").unwrap();
    }
    NAME_REGEX.captures(&string).expect("Could not find game name").get(1).expect("Could not find game name").as_str()
}

fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let app = clap::App::from_yaml(yaml).setting(clap::AppSettings::ArgRequiredElseHelp);
    let cmd_main = &app.get_matches();

    if cmd_main.subcommand_matches("ping").is_some() {
        println!("Pong!")
    }

    else if cmd_main.subcommand_matches("network").is_some() {
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

    else if let Some(cmd_steam) = cmd_main.subcommand_matches("steam") {

        if let Some(cmd_list) = cmd_steam.subcommand_matches("list") {
            let steamapps_path = match cmd_list.value_of("steamapps") {
                None => {
                    get_default_steamapps()
                },
                Some(string) => {
                    Path::new(string)
                },
            };

            if !steamapps_path.is_dir() {
                panic!("The steamapps path is not a directory.")
            }

            for manifest in find_manifests_in_folder(steamapps_path) {
                let path = &manifest.path();
                let manifest_contents = fs::read_to_string(&path).expect("Could not read manifest");

                let appid = extract_appid_from_manifest_contents(&manifest_contents);
                let game_name = extract_game_name_from_manifest_contents(&manifest_contents);

                println!("{} - {}", &appid, &game_name);
            };
        }

        else if let Some(cmd_backup) = cmd_steam.subcommand_matches("backup") {
            println!("backup");
        }

        else if let Some(cmd_restore) = cmd_steam.subcommand_matches("restore") {
            println!("restore");
        }
    }
}
