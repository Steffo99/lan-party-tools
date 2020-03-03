//! The module handling `lan-party-tools steam`.

pub mod appmanifest;
pub mod steamapps;

use std::io;
use std::fs;
use std::path::Path;
use steamapps::SteamApps;
use clap;
use fs_extra;
use crate::steam::appmanifest::AppManifest;

/// Find all manifests in the specified folder and return a [`Vec`] containing them as [`fs::DirEntry`]ies.
fn find_manifests_in_folder(folder: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut manifests: Vec<fs::DirEntry> = vec![];
    for object in fs::read_dir(folder)? {
        let object = object?;

        if !object.path().is_file() {
            continue;
        }

        if !object.file_name().to_str().unwrap().starts_with("appmanifest_") {
            continue;
        }

        manifests.push(object);
    };
    Ok(manifests)
}

/// The function handling `lan-party-tools steam list`.
pub fn steam_list_command(steamapps: Option<&str>) -> Result<(), &'static str> {
    let steamapps = SteamApps::from_console_input(&steamapps);

    for manifest in find_manifests_in_folder(&steamapps.location).ok().ok_or("Could not find appmanifests")? {
        let path = &manifest.path();
        let manifest = appmanifest::AppManifest::new(&path).ok().ok_or("Could not read appmanifest")?;
        let appid = &manifest.appid().ok_or("Could not find appid")?;
        let game_name = &manifest.game_name().ok_or("Could not find game name")?;
        println!("{}\t- {}", &appid, &game_name);
    };

    Ok(())
}

/// Copy a `appmanifest_XXX.acf` file from a source to a destination.
fn copy_manifest(from: &Path, to: &Path) -> Result<u64, fs_extra::error::Error> {
    fs_extra::copy_items(&vec![from], &to, &fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 0,
        copy_inside: true,
        depth: 0
    })
}

/// Calculate the current file copy progress and return it in a format [`pbr`] can understand.
///
/// ```
/// assert_eq!(progress_percentage(1, 2), 5000);
/// assert_eq!(progress_percentage(1, 10000), 1);
/// ```
fn progress_percentage(num: u64, den: u64) -> u64 {
    return (((num as f64) / (den as f64)) * 10000f64) as u64;
}

/// Recursively copy the contents of a `installdir` from a source to a destination, and display a progress bar describing the copy status.
fn copy_game_files(from: &Path, to: &Path, message: &str) -> Result<u64, fs_extra::error::Error> {
    let mut game_files_pb = pbr::ProgressBar::new(10000);
    game_files_pb.format("|█▓░|");
    game_files_pb.show_counter = false;
    game_files_pb.show_speed = false;
    game_files_pb.message(&message);
    let result = fs_extra::copy_items_with_progress(&vec![from], &to, &fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 1_048_576,
        copy_inside: true,
        depth: 0
    }, |process_info: fs_extra::TransitProcess| {
        game_files_pb.set(progress_percentage(process_info.copied_bytes, process_info.total_bytes));
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    });
    game_files_pb.finish_print("\n");
    result
}

/// The function handling `lan-party-tools steam backup`.
pub fn steam_backup_command(
    steamapps: Option<&str>,
    destination: Option<&str>,
    appids: Option<clap::Values>
) -> Result<(), &'static str> {

    let steam = SteamApps::from_console_input(&steamapps);
    let steam_common = &steam.get_common().ok_or("No common folder found in steamapps")?;

    let destination = SteamApps::from_console_input(&destination);
    let destination_common = &destination.get_or_create_common().ok().ok_or("Could not create common folder in destination")?;

    if appids.is_none() {
        return Err("Nothing to backup");
    }
    for appid in appids.unwrap() {
        let manifest_path = &steam.get_manifest_path(&appid);
        let manifest = AppManifest::new(&manifest_path).ok().ok_or("Could not read appmanifest")?;
        let steam_installdir = &manifest.get_installdir(&steam_common).ok_or("Could not find installdir")?;

        copy_manifest(&manifest_path, &destination.location).ok().ok_or("Couldn't copy manifest")?;
        copy_game_files(&steam_installdir, &destination_common, &appid).ok().ok_or("Couldn't copy game files")?;
    };
    Ok(())
}

/// The function handling `lan-party-tools steam restore`.
pub fn steam_restore_command(
    steamapps: Option<&str>,
    source: Option<&str>,
    appids: Option<clap::Values>
) -> Result<(), &'static str> {

    let source = SteamApps::from_console_input(&source);
    let source_common = &source.get_common().ok_or("No common folder found in destination")?;

    let steam = SteamApps::from_console_input(&steamapps);
    let steam_common = &steam.get_or_create_common().ok().ok_or("Could not create common folder in steamapps")?;

    if appids.is_none() {
        return Err("Nothing to restore");
    }
    for appid in appids.unwrap() {
        let manifest_path = &source.get_manifest_path(&appid);
        let manifest = AppManifest::new(&manifest_path).ok().ok_or("Could not read appmanifest for appid")?;
        let source_installdir = &manifest.get_installdir(&source_common).ok_or("Could not find installdir")?;

        copy_manifest(&manifest_path, &steam.location).ok().ok_or("Couldn't copy manifest")?;
        copy_game_files(&source_installdir, &steam_common, &appid).ok().ok_or("Couldn't copy game files")?;
    };
    Ok(())
}
