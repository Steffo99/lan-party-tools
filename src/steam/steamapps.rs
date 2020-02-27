use std::path::Path;
use std::fs;
use std::io;
use super::appmanifest::AppManifest;
use clap::App;

pub struct SteamApps<'a> {
    pub location: &'a Path
}

impl SteamApps {
    pub fn default() -> Self<'static> {
        Self {
            location: {
                if cfg!(windows) {
                    Path::new(r"C:\Program Files (x86)\Steam\steamapps")
                } else if cfg!(macos) {
                    Path::new(r"~/Library/Application Support/Steam")
                } else if cfg!(linux) {
                    Path::new(r"~/.steam/steam/steamapps")
                } else {
                    unimplemented!("Unsupported platform!")
                }
            }
        }
    }

    pub fn path(location: &Path) -> Self {
        Self {
            location
        }
    }

    pub fn from_console_input(input: &Option<&str>) -> Self {
        match input {
            None => {
                Self::default()
            },
            Some(string) => {
                Self::path(Path::new(string))
            }
        }
    }

    pub fn get_common(&self) -> Option<&Path> {
        let path = &self.location.join(Path::new("common"));

        if ! &path.is_dir() {
            return None
        };

        Some(Path)
    }

    pub fn get_or_create_common(&self) -> io::Result<&Path> {
        let path = &self.location.join(Path::new("common"));

        if ! &path.is_dir() {
            fs::create_dir(&destination_common_path)?;
        }

        Ok(path)
    }

    pub fn get_manifest(&self, appid: &str) -> io::Result<AppManifest> {
        let path = &self.location.join(Path::new(&format!("appmanifest_{}.acf", &appid)));
        AppManifest::new(path)
    }
}

