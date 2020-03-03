//! The module containing the [`SteamApps`] struct.

use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use super::appmanifest::AppManifest;

/// A struct representing a `steamapps` folder / a steam game library folder.
pub struct SteamApps<'a> {
    pub location: &'a Path
}

impl<'a> SteamApps<'a> {
    /// Create a new [`SteamApps`] from its default location on each platform:
    ///
    /// - `C:\Program Files (x86)\Steam\steamapps` on Windows
    /// - `~/Library/Application Support/Steam` on Mac OSX
    /// - `~/.steam/steam/steamapps` on Linux
    ///
    /// # Panics
    ///
    /// If the platform does not match any of the ones listed before.
    pub fn default() -> Self {
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

    /// Create a new [`SteamApps`] from a [`Path`].
    pub fn path(location: &'a Path) -> Self {
        Self {
            location
        }
    }

    /// Create a new [`SteamApps`] from a [`Option`], using the default location if the [`Option`] is `None` and interpreting the passed string as a Path if the [`Option`] is `Some`.
    pub fn from_console_input(input: &'a Option<&str>) -> Self {
        match input {
            None => {
                Self::default()
            },
            Some(string) => {
                Self::path(Path::new(string))
            }
        }
    }

    /// Get the path to the `steamapps/common` folder, where the `installdir`s are usually located.
    ///
    /// # Returns
    /// - [`Option::None`] if the `common` directory does not exist
    /// - [`Option::Some`] otherwise
    pub fn get_common(&self) -> Option<PathBuf> {
        let path = self.location.join(Path::new("common"));

        if ! &path.is_dir() {
            return None
        };

        Some(path)
    }

    /// Get the path to the `steamapps/common` folder, where the `installdir`s are usually located;
    /// if the `common` folder does not exist, try to create it.
    ///
    /// # Returns
    /// - [`Result::Err`] if the `common` directory could not be created
    /// - [`Result::Ok`] otherwise
    pub fn get_or_create_common(&self) -> io::Result<PathBuf> {
        let path = self.location.join(Path::new("common"));

        if ! &path.is_dir() {
            fs::create_dir(&path)?;
        }

        Ok(path)
    }

    /// Get the [`PathBuf`] to the `appmanifest_XXX.acf` file for the app with the id `appid`.
    pub fn get_manifest_path(&self, appid: &str) -> PathBuf {
        self.location.join(Path::new(&format!("appmanifest_{}.acf", &appid)))
    }

    /// Get the [`AppManifest`] for the app with the id `appid`.
    pub fn get_manifest(&self, appid: &str) -> io::Result<AppManifest> {
        AppManifest::new(&self.get_manifest_path(&appid))
    }
}

