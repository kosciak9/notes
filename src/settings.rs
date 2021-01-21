use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub daemon: bool,
    pub directory: String,
    pub index: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("/home/kosciak/.config/slipbox/config"))?;

        s.try_into()
    }
}
