#![feature(str_strip)]

use clap;
use std::path::Path;
use std::fs;
use std::io;
use lazy_static::lazy_static;
use fs_extra::dir::CopyOptions;
use pbr;

mod network;
mod steam;


fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let app = clap::App::from_yaml(yaml).setting(clap::AppSettings::ArgRequiredElseHelp);
    let cmd_main = &app.get_matches();

    if cmd_main.subcommand_matches("ping").is_some() {
        println!("Pong!")
    }

    else if cmd_main.subcommand_matches("network").is_some() {
        network::network_command();
    }

    else if let Some(cmd_steam) = cmd_main.subcommand_matches("steam") {

        if let Some(cmd_list) = cmd_steam.subcommand_matches("list") {
            steam::steam_list_command(&cmd_list.value_of("steamapps"))
        }

        else if let Some(cmd_backup) = cmd_steam.subcommand_matches("backup") {
            steam::steam_backup_command(
                &cmd_backup.value_of("steamapps"),

            )
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
            for appid in appids {
                // Find the game manifest
                let manifest_path = &source_path.join(Path::new());
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
                    buffer_size: 1_048_576,
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
                let mut game_files_pb = pbr::ProgressBar::new(10000);
                game_files_pb.format("|█▓░|");
                game_files_pb.show_counter = false;
                game_files_pb.show_speed = false;
                game_files_pb.message(&appid);
                match fs_extra::copy_items_with_progress(&vec![installdir_path], &steamapps_common_path, &CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 1_048_576,
                    copy_inside: true,
                    depth: 0
                }, |process_info: fs_extra::TransitProcess| {
                    game_files_pb.set(percentage(process_info.copied_bytes, process_info.total_bytes));
                    fs_extra::dir::TransitProcessResult::ContinueOrAbort
                }) {
                    Err(_) => {
                        eprintln!("{}\t! Error copying game files", &appid);
                        continue;
                    },
                    _ => {},
                };
                game_files_pb.finish_print("\n");

            }
        }
    }
}
