use regex;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use lazy_static::lazy_static;

pub struct AppManifest {
    contents: String
}

impl AppManifest {
    pub fn new(path: &Path) -> io::Result<Self> {
        Ok(Self {
            contents: fs::read_to_string(&path)?
        })
    }

    pub fn appid(&self) -> Option<&str> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"appid\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(REGEX.captures(&self.contents)?.get(1)?.as_str())
    }

    pub fn game_name(&self) -> Option<&str> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"name\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(REGEX.captures(&self.contents)?.get(1)?.as_str())
    }

    pub fn installdir(&self) -> Option<&Path> {
        lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new("(?m)^\\s*\"installdir\"\\s+\"(.+)\"\\s*$").unwrap();
        }
        Some(Path::new(REGEX.captures(&self.contents)?.get(1)?.as_str()))
    }

    pub fn get_installdir(&self, base_dir: &Path) -> Option<PathBuf> {
        let installdir = base_dir.join(&self.installdir()?);

        if ! &installdir.is_dir() {
            return None
        };

        Some(installdir)
    }
}
