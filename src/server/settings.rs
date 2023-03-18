use ws::Settings;
use std::io::{self};
use crate::config::{self,IoConfig};

// Wrapper around ws::Settings to only let some be user defined.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(default)]
pub struct ServerConfig {
    pub max_connections: usize,
    pub queue_size: usize,
    //pub panic_on_new_connection: bool,
    //pub panic_on_shutdown: bool,
    pub fragments_capacity: usize,
    pub fragments_grow: bool,
    pub fragment_size: usize,
    pub in_buffer_capacity: usize,
    pub in_buffer_grow: bool,
    pub out_buffer_capacity: usize,
    pub out_buffer_grow: bool,
    //pub panic_on_internal: bool,
    //pub panic_on_capacity: bool,
    //pub panic_on_protocol: bool,
    //pub panic_on_encoding: bool,
    //pub panic_on_queue: bool,
    //pub panic_on_io: bool,
    //pub panic_on_timeout: bool,
    //pub shutdown_on_interrupt: bool,
    pub masking_strict: bool,
    pub key_strict: bool,
    pub method_strict: bool,
    //pub encrypt_server: bool,
    pub tcp_nodelay: bool,
}

const SERVER_CONFIG_FILENAME: &'static str = "./server.config";

impl ServerConfig {
    pub fn to_disk(&self) -> io::Result<()> {
        self.save_to_disk()
    }
    pub fn from_disk() -> Self {
        if config::active().skip_load_server_settings {
            Self::default()
        } else {
            ServerConfig::load_from_disk()
        }
    }
}

impl config::IoConfig<'static> for ServerConfig {
    fn file() -> &'static str {
        "./server.config"
    }
    fn filename() -> &'static str {
        "server.config"
    }
}


impl Default for ServerConfig {
    fn default() -> Self {
        let def = Settings::default();
        ServerConfig {
            max_connections: def.max_connections,
            queue_size: def.queue_size,
            fragments_capacity: def.fragments_capacity,
            fragments_grow: def.fragments_grow,
            fragment_size: def.fragment_size,
            in_buffer_capacity: def.in_buffer_capacity,
            in_buffer_grow: def.in_buffer_grow,
            out_buffer_capacity: def.out_buffer_capacity,
            out_buffer_grow: def.out_buffer_grow,
            masking_strict: def.masking_strict,
            key_strict: def.key_strict,
            method_strict: def.method_strict,
            tcp_nodelay: def.tcp_nodelay,
        }
    }
}
impl Into<Settings> for ServerConfig {
    fn into(self) -> Settings {
        Settings {
            max_connections: self.max_connections,
            queue_size: self.queue_size,
            panic_on_new_connection: false,
            panic_on_shutdown: false,
            fragments_capacity: self.fragments_capacity,
            fragments_grow: self.fragments_grow,
            fragment_size: self.fragment_size,
            max_fragment_size: 256,
            in_buffer_capacity: self.in_buffer_capacity,
            in_buffer_grow: self.in_buffer_grow,
            out_buffer_capacity: self.out_buffer_capacity,
            out_buffer_grow: self.out_buffer_grow,
            panic_on_internal: false,
            panic_on_capacity: false,
            panic_on_protocol: false,
            panic_on_encoding: false,
            panic_on_queue: false,
            panic_on_io: false,
            panic_on_timeout: false,
            shutdown_on_interrupt: false,
            masking_strict: self.masking_strict,
            key_strict: self.key_strict,
            method_strict: self.method_strict,
            encrypt_server: false,
            tcp_nodelay: self.tcp_nodelay,
        }
    }
}
