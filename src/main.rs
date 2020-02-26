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
use fs_extra::dir::CopyOptions;


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

fn find_manifests_in_folder(folder: &Path) -> Vec<fs::DirEntry> {
    let mut manifests: Vec<fs::DirEntry> = vec![];
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

struct AppManifest {
    contents: String
}

impl AppManifest {
    fn new(path: &Path) -> io::Result<Self> {
        Ok(Self {
            contents: fs::read_to_string(&path)?
        })
    }

    fn appid(&self) -> Option<&str> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"appid\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(REGEX.captures(&self.contents)?.get(1)?.as_str())
    }

    fn game_name(&self) -> Option<&str> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"name\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(REGEX.captures(&self.contents)?.get(1)?.as_str())
    }

    fn installdir(&self) -> Option<&str> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"installdir\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(REGEX.captures(&self.contents)?.get(1)?.as_str())
    }
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
            // Find steamapps and ensure it is a directory
            let steamapps_path = match cmd_list.value_of("steamapps") {
                None => {
                    get_default_steamapps()
                },
                Some(string) => {
                    Path::new(string)
                },
            };
            if !steamapps_path.is_dir() {
                clap::Error {
                    message: "steamapps path is not a directory".to_string(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: Some(vec!["steamapps".to_string()]),
                }.exit();
            }
            // Find all appmanifests in steamapps
            for manifest in find_manifests_in_folder(&steamapps_path) {
                let path = &manifest.path();
                let manifest = match AppManifest::new(&path) {
                    Ok(m) => m,
                    Err(_) => {
                        eprintln!("{}\t! Could not read appmanifest", &path.file_name().unwrap().to_str().unwrap());
                        return;
                    },
                };
                let appid = match manifest.appid() {
                    Some(m) => m,
                    None => {
                        eprintln!("{}\t! Could not find appid", &path.file_name().unwrap().to_str().unwrap());
                        return;
                    },
                };
                let game_name = match manifest.game_name() {
                    Some(m) => m,
                    None => {
                        eprintln!("{}\t! Could not find game name", &appid);
                        return;
                    },
                };
                println!("{}\t- {}", &appid, &game_name);
            };
        }

        else if let Some(cmd_backup) = cmd_steam.subcommand_matches("backup") {
            // Find steamapps and ensure it is a directory
            let steamapps_path = match cmd_backup.value_of("steamapps") {
                None => {
                    get_default_steamapps()
                },
                Some(string) => {
                    Path::new(string)
                },
            };
            if !steamapps_path.is_dir() {
                clap::Error {
                    message: "steamapps path is not a directory".to_string(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: Some(vec!["steamapps".to_string()]),
                }.exit();
            }

            // Find steamapps/common and ensure it is a directory
            let steamapps_common_path = &steamapps_path.join(Path::new("common"));
            if !steamapps_common_path.is_dir() {
                eprintln!("Error: No common folder in steamapps");
                return;
            }

            // Find the appids to parse
            let appids = cmd_backup.values_of("appids").unwrap();

            // Find the backup destination
            let destination_path = match cmd_backup.value_of("destination") {
                None => {
                    Path::new(".")
                },
                Some(destination) => {
                    Path::new(destination)
                },
            };

            // Find (or create) the destination/common directory
            let destination_common_path = &destination_path.join(Path::new("common"));
            if ! &destination_common_path.is_dir() {
                match fs::create_dir(&destination_common_path) {
                    Err(_) => {
                        eprintln!("Warning: Could not create directory at {}", &destination_common_path.to_str().unwrap());
                    },
                    _ => {}
                };
            }

            // Backup games
            for appid in appids {
                // Find the game manifest
                let manifest_path = &steamapps_path.join(Path::new(&format!("appmanifest_{}.acf", &appid)));
                let manifest = match AppManifest::new(&manifest_path) {
                    Ok(am,) => am,
                    Err(error) => {
                        eprintln!("{}\t! Could not read {} ({})", &appid, &manifest_path.to_str().unwrap(), &error.to_string());
                        continue;
                    },
                };

                // Find manifest elements
                let installdir = match manifest.installdir() {
                    Some(m) => m,
                    None => {
                        eprintln!("{}\t! Could not find installdir", &appid);
                        return;
                    },
                };

                // Find the installdir folder
                let installdir_path = &steamapps_common_path.join(Path::new(installdir));
                if !installdir_path.is_dir() {
                    eprintln!("{}\t! installdir is not a folder", &appid);
                    return;
                }

                // Copy the manifest
                match fs_extra::copy_items(&vec![manifest_path], &destination_path, &CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 0,
                    copy_inside: true,
                    depth: 0
                }) {
                    Err(_) => {
                        eprintln!("{}\t! Error copying manifest", &appid);
                        continue;
                    },
                    _ => {},
                };

                // Copy the game files
                match fs_extra::copy_items(&vec![installdir_path], &destination_common_path, &CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 0,
                    copy_inside: true,
                    depth: 0
                }) {
                    Err(_) => {
                        eprintln!("{}\t! Error copying game files", &appid);
                        continue;
                    },
                    _ => {},
                };

                println!("{}\t- Successfully backed up", &appid);
            }
        }

        else if let Some(cmd_restore) = cmd_steam.subcommand_matches("restore") {
            // Find steamapps and ensure it is a directory
            let steamapps_path = match cmd_restore.value_of("steamapps") {
                None => {
                    get_default_steamapps()
                },
                Some(string) => {
                    Path::new(string)
                },
            };
            if !steamapps_path.is_dir() {
                clap::Error {
                    message: "steamapps path is not a directory".to_string(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: Some(vec!["steamapps".to_string()]),
                }.exit();
            }

            // Find steamapps/common and ensure it is a directory
            let steamapps_common_path = &steamapps_path.join(Path::new("common"));
            if !steamapps_common_path.is_dir() {
                eprintln!("Error: No common folder in steamapps");
                return;
            }

            // Find the appids to parse
            let appids = cmd_restore.values_of("appids").unwrap();

            // Find the backup source and ensure it is a directory
            let source_path = match cmd_restore.value_of("source") {
                None => {
                    Path::new(".")
                },
                Some(destination) => {
                    Path::new(destination)
                },
            };
            if !source_path.is_dir() {
                clap::Error {
                    message: "source path is not a directory".to_string(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: Some(vec!["source".to_string()]),
                }.exit();
            }

            // Find the destination/common directory
            let source_common_path = &source_path.join(Path::new("common"));
            if ! &source_common_path.is_dir() {
                clap::Error {
                    message: "no common directory in source path".to_string(),
                    kind: clap::ErrorKind::InvalidValue,
                    info: Some(vec!["source".to_string()]),
                }.exit();
            }

            // Restore backups
            for appid in appids {// Find the game manifest
                let manifest_path = &source_path.join(Path::new(&format!("appmanifest_{}.acf", &appid)));
                let manifest = match AppManifest::new(&manifest_path) {
                    Ok(am,) => am,
                    Err(error) => {
                        eprintln!("{}\t! Could not read {} ({})", &appid, &manifest_path.to_str().unwrap(), &error.to_string());
                        continue;
                    },
                };

                // Find manifest elements
                let installdir = match manifest.installdir() {
                    Some(m) => m,
                    None => {
                        eprintln!("{}\t! Could not find installdir", &appid);
                        return;
                    },
                };

                // Find the installdir folder
                let installdir_path = &source_common_path.join(Path::new(installdir));
                if !installdir_path.is_dir() {
                    eprintln!("{}\t! installdir is not a folder", &appid);
                    return;
                }

                // Copy the manifest
                match fs_extra::copy_items(&vec![manifest_path], &steamapps_path, &CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 0,
                    copy_inside: true,
                    depth: 0
                }) {
                    Err(_) => {
                        eprintln!("{}\t! Error copying manifest", &appid);
                        continue;
                    },
                    _ => {},
                };

                // Copy the game files
                match fs_extra::copy_items(&vec![installdir_path], &steamapps_common_path, &CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 0,
                    copy_inside: true,
                    depth: 0
                }) {
                    Err(_) => {
                        eprintln!("{}\t! Error copying game files", &appid);
                        continue;
                    },
                    _ => {},
                };

                println!("{}\t- Successfully restored", &appid);

            }
        }
    }
}
