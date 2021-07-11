use eyre::{Report, WrapErr};
use rumqttc::{ClientConfig, MqttOptions, Transport};
use serde_derive::Deserialize;
use std::fs::read_to_string;

const CONFIG_FILENAME: &str = "hovercontrol.toml";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub mqtt: Option<MqttConfig>,
}

impl Config {
    pub fn from_file() -> Result<Config, Report> {
        Config::read(CONFIG_FILENAME)
    }

    fn read(filename: &str) -> Result<Config, Report> {
        let config_file =
            read_to_string(filename).wrap_err_with(|| format!("Reading {}", filename))?;
        Ok(toml::from_str(&config_file)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub use_tls: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_name: Option<String>,
}

/// Construct the `MqttOptions` for connecting to the MQTT broker based on configuration options or
/// defaults.
pub fn get_mqtt_options(config: MqttConfig, device_id: &str) -> MqttOptions {
    let client_name = config.client_name.unwrap_or_else(|| device_id.to_owned());

    let mut mqtt_options = MqttOptions::new(client_name, config.host, config.port);

    mqtt_options.set_keep_alive(5);
    if let (Some(username), Some(password)) = (config.username, config.password) {
        mqtt_options.set_credentials(username, password);
    }

    if config.use_tls {
        let mut client_config = ClientConfig::new();
        client_config.root_store =
            rustls_native_certs::load_native_certs().expect("could not load platform certs");
        mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));
    }
    mqtt_options
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parsing the example config file should not give any errors.
    #[test]
    fn example_config() {
        Config::read("hovercontrol.example.toml").unwrap();
    }

    /// Parsing an empty config file should not give any errors.
    #[test]
    fn empty_config() {
        toml::from_str::<Config>("").unwrap();
    }
}
