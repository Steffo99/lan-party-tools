#![feature(ip)]
#![feature(str_strip)]

use clap;
use systemstat::{Platform, NetworkAddrs, IpAddr};
use reqwest;
use std::path::Path;
use std::fs;
use std::io;
use fs_extra;
use std::io::Write;

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

fn create_manifest(directory: &Path, appid: &str, installdir: &Path) -> io::Result<()> {
    let path = directory.join(Path::new(&format!("appmanifest_{}.acf", appid)));
    println!("{:?}", path);

    let mut file = fs::File::create(path)?;
    file.write(b"\"AppState\"\n")?;
    file.write(b"{\n")?;
    file.write(format!("\t\"appid\"\t\t\"{}\"\n", appid).as_bytes())?;
    file.write(b"\t\"Universe\"\t\t\"1\"\n")?;
    file.write(b"\t\"StateFlags\"\t\t\"1026\"\n")?;
    file.write(b"\t\"LastUpdated\"\t\t\"0\"\n")?;
    file.write(b"\t\"UpdateResult\"\t\t\"4\"\n")?;
    file.write(b"\t\"SizeOnDisk\"\t\t\"0\"\n")?;
    file.write(b"\t\"buildid\"\t\t\"0\"\n")?;
    file.write(b"\t\"BytesToDownload\"\t\t\"0\"\n")?;
    file.write(b"\t\"BytesDownloaded\"\t\t\"0\"\n")?;
    file.write(format!("\t\"installdir\"\t\t\"{}\"\n", installdir.file_name().unwrap().to_str().unwrap()).as_bytes())?;
    file.write(b"}")?;

    return Ok(());
}

fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let app = clap::App::from_yaml(yaml).setting(clap::AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();

    if matches.subcommand_matches("ping").is_some() {
        println!("Pong!")
    }

    else if matches.subcommand_matches("network").is_some() {
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

    else if let Some(steammv) = matches.subcommand_matches("steamcp") {
        let source_arg = steammv.value_of("source").expect("Expected a source path.");
        let destination_arg = steammv.value_of("steamapps");
        let steamapps = match destination_arg {
            None => {
                if cfg!(windows) {
                    Path::new(r"C:\Program Files (x86)\Steam\steamapps")
                } else {
                    unimplemented!("Non-Windows OSes are not supported yet");
                }
            },
            Some(arg) => {
                Path::new(arg)
            },
        };
        let steamapps_common = steamapps.join(Path::new("common"));
        let source = Path::new(source_arg);

        let appid_arg = steammv.value_of("appid");
        let raw_appid = match appid_arg {
            None => {
                println!("Trying to find the steamid of the game...");
                let steam_appid_path = &source.join(Path::new("steam_appid.txt"));
                fs::metadata(&steam_appid_path).expect("No steam_appid.txt file found.");
                String::from_utf8(fs::read(&steam_appid_path).expect("Couldn't read steam_appid.txt file.")).expect("Failed to parse steam_appid.txt file.")
            },
            Some(id) => {id.to_string()},
        };
        let appid = raw_appid.strip_suffix("\n").unwrap_or(raw_appid.as_str());
        println!("Detected appid: {}", &appid);

        println!("Copying...");
        fs_extra::copy_items(&vec![&source], &steamapps_common, &fs_extra::dir::CopyOptions {
            overwrite: false,
            skip_exist: true,
            buffer_size: 0,
            copy_inside: false,
            depth: 0
        }).unwrap();
        println!("Copy successful!");

        println!("Generating manifest...");
        create_manifest(&steamapps, &appid, &source).expect("Could not create manifest file.");
        println!("Manifest created!");
    }
}
