use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub motd: String,
    pub use_default_config: bool,
    pub skip_load_server_settings: bool,
    pub server_settings_file: String,
    pub player_count: usize,
    pub turn_limit: u32,
}
impl Default for Config {
    fn default() -> Config {
        Config {
            motd: String::from("Hello World!"),
            use_default_config: true,
            skip_load_server_settings: true,
            server_settings_file: String::from("./server.config"),
            player_count: 2,
            turn_limit: 3,
        }
    }
}

pub trait IoConfig<'a>: DeserializeOwned + Serialize + Default + Sized {
    /// The absolute or reltive path for the config file. ex. "./file.config"
    fn file() -> &'a str;
    /// A user friendly name for the config file.
    fn filename() -> &'a str;
    /// Saves this Config to the filesystem.
    fn save_to_disk(&self) -> Result<()> {
        let writer = File::create(Self::file())?;
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }
    /// Reads the settings.config from the filesystem. Creates a new default file if no file exists.
    fn load_from_disk() -> Self {
        let name = Self::filename();

        match Self::try_load_from_disk() {
            Ok(s) => {
                info!("Done loading '{}' file.", name);
                s
            }
            Err(e) => match e.kind() {
                ErrorKind::InvalidData => {
                    warn!("Invalid JSON in '{}' file: {}", name, e);
                    Self::default()
                }
                ErrorKind::NotFound => {
                    info!("Creating new '{}' file.", name);
                    let default_config = Self::default();
                    if let Err(e) = default_config.save_to_disk() {
                        warn!("IO Error when creating new '{}' file: {}", name, e)
                    }
                    default_config
                }
                _ => {
                    warn!("Could not read settings from '{}': {}", name, e);
                    Self::default()
                }
            },
        }
    }
    /// Tries to reads the settings.config from the filesystem.
    /// Errors can be any File::open() err or an ErrorKind::InvalidData if the file's json is invalid.
    fn try_load_from_disk() -> Result<Self> {
        let reader = File::open(Self::file())?;
        match serde_json::from_reader(reader) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
}
impl IoConfig<'static> for Config {
    fn file() -> &'static str {
        "./settings.config"
    }
    fn filename() -> &'static str {
        "settings.config"
    }
}

lazy_static! {
    static ref STATIC_CONFIG: Config = Config::load_from_disk();
    //static ref CONFIG: Mutex<Arc<Config>> = Mutex::new(Arc::new(Config::default()));
}
/// Gets the static never changing Config.
pub fn active() -> &'static Config {
    &STATIC_CONFIG
}

// /// Gets the current active runtime config.
// pub fn active() -> Arc<Config> {
    // CONFIG.lock().unwrap().clone()
// }
// /// Sets the new active config. Other configs are not changed unless active() is called again.
// pub fn set_active(config: Config) -> Arc<Config> {
    // let ret = Arc::new(config);
    // *CONFIG.lock().unwrap() = ret.clone();
    // ret
// }
