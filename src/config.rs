use serde_json;
use serde;
use std::sync::Arc;
use std::sync::Mutex;
use std::{fs, io};
use std::path::Path;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub use_default_config: bool,
    pub skip_load_server_settings: bool,
}
impl Default for Config {
    fn default() -> Config {
        Config {
            use_default_config: true,
            skip_load_server_settings: true,
        }
    }
}

pub trait IoConfig: serde::de::DeserializeOwned + serde::Serialize + Default +Sized {
    const FILE: &'static AsRef<Path>;
    const FILENAME: &'static str;
    // saves this Config to the filesystem.
    fn write_to_disk(&self) -> io::Result<()> {
        let writer = fs::File::create(Self::FILE)?;
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }
    // reads the settings.config form the filesystem. Creates a new default file if no file exists.
    fn load_from_disk() -> Self {
        match fs::File::open(Self::FILE) {
            Ok(reader) => match serde_json::from_reader(reader) {
                Ok(s) => {
                    info!("Done loading '{}' file.", Self::FILENAME);
                    s
                }
                Err(e) => {
                    warn!("Invalid JSON in '{}' file: {}", Self::FILENAME, e);
                    warn!("Default settings.config will be used.");
                    Self::default()
                }
            },
            Err(e) => {
                let ret = if e.kind() == io::ErrorKind::NotFound {
                    info!("Creating new '{}' file.", Self::FILENAME);
                    let res = Self::default();
                    if let Err(e) = res.write_to_disk() {
                        warn!("IO Error when creating new '{}' file: {}", Self::FILENAME, e)
                    }
                    res
                } else {
                    warn!("Could not read settings from '{}': {}", Self::FILENAME, e);
                    Self::default()
                };
                info!("Done loading '{}' file. (default is being used)", Self::FILENAME);
                ret
            }
        }
    }
}
impl IoConfig for Config {
    const FILE: &'static AsRef<Path> = &"./settings.config";
    const FILENAME: &'static str = "settings.config";
}

lazy_static! {
    static ref CONFIG: Mutex<Arc<Config>> = Mutex::new(Arc::new(Config::default()));
}

// Gets the current active runtime config.
pub fn active() -> Arc<Config> {
    CONFIG.lock().unwrap().clone()
}
// Sets the new active config. Other configs are not changed unless active() is called again.
pub fn set_active(config: Config) -> Arc<Config> {
    let ret = Arc::new(config);
    *CONFIG.lock().unwrap() = ret.clone();
    ret
}
