pub mod appmanifest;
pub mod steamapps;

use std::io;
use std::fs;
use std::path::Path;
use steamapps::SteamApps;
use clap;
use fs_extra;


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

pub fn steam_list_command(steamapps: &Option<&str>) {
    let steamapps = SteamApps::from_console_input(&steamapps);

    for manifest in find_manifests_in_folder(&steamapps.location) {
        let path = &manifest.path();
        let manifest = match appmanifest::AppManifest::new(&path) {
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

fn copy_manifest(from: &Path, to: &Path) -> Result<u64, fs_extra::error::Error> {
    fs_extra::copy_items(&vec![from], &to, &fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 0,
        copy_inside: true,
        depth: 0
    })
}

fn progress_percentage(num: u64, den: u64) -> u64 {
    return (((num as f64) / (den as f64)) * 10000f64) as u64;
}


fn copy_game_files(from: &Path, to: &Path) -> Result<u64, fs_extra::error::Error> {
    let mut game_files_pb = pbr::ProgressBar::new(10000);
    game_files_pb.format("|█▓░|");
    game_files_pb.show_counter = false;
    game_files_pb.show_speed = false;
    game_files_pb.message(&appid);
    let result = fs_extra::copy_items_with_progress(&vec![from], &to, &fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 1_048_576,
        copy_inside: true,
        depth: 0
    }, |process_info: fs_extra::TransitProcess| {
        game_files_pb.set(percentage(process_info.copied_bytes, process_info.total_bytes));
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    });
    game_files_pb.finish_print("\n");
    result
}

pub fn steam_backup_command(
    steamapps: &Option<&str>,
    destination: &Option<&str>,
    appids: clap::Values
) -> Result<(), &'static str> {

    let steamapps = SteamApps::from_console_input(&steamapps);
    let steam_common = &steamapps.get_common().ok_or(Err("No common folder found in steamapps"))?;

    let destination = SteamApps::from_console_input(&destination);
    let destination_common = &destination.get_or_create_common().ok_or(Err("Could not create common folder in destination"))?;

    for appid in appids {
        let manifest = &steamapps.get_manifest(&appid);
        let installdir = &manifest.get_installdir().ok_or(Err(format!("Could not find installdir for appid {}", &appid)))?;

        copy_manifest(&/*TODO*/, &destination.location);

        // Copy the game files
    };
    Ok(())
}