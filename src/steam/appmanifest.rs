use regex;
use std::path::Path;
use std::io;
use std::fs;

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

    pub fn get_installdir(&self, base_dir: &Path) -> Option<&Path> {
        let installdir = &self.installdir()?;

        if ! &path.is_dir() {
            return None
        };

        Some(installdir)
    }
}