use crate::controller::{
    DEFAULT_MAX_SPEED, DEFAULT_SCALE, DEFAULT_SPRING_CONSTANT, MAX_MAX_SPEED, MAX_SCALE,
    MAX_SPRING_CONSTANT, SPRING_CONSTANT_STEP,
};
use eyre::Report;
use homie_device::{HomieDevice, Node, Property};
use log::error;
use messages::{Side, SpeedLimits};
use rumqttc::{ClientConfig, MqttOptions, Transport};
use tokio::runtime::Handle;

const MQTT_HOST: &str = "test.mosquitto.org";
const MQTT_PORT: u16 = 1883;
const USE_TLS: bool = false;
const MQTT_USERNAME: Option<&str> = None;
const MQTT_PASSWORD: Option<&str> = None;
const HOMIE_PREFIX: &str = "homie";
const HOMIE_DEVICE_ID: &str = "hoverkite";
const HOMIE_DEVICE_NAME: &str = "Hoverkite";

pub struct Homie<'a> {
    homie: HomieDevice,
    handle: &'a Handle,
}

impl<'a> Homie<'a> {
    pub async fn make_homie_device(handle: &'a Handle) -> Result<Homie<'a>, Report> {
        let mqtt_options = get_mqtt_options();
        let device_base = format!("{}/{}", HOMIE_PREFIX, HOMIE_DEVICE_ID);
        let homie_builder = HomieDevice::builder(&device_base, HOMIE_DEVICE_NAME, mqtt_options);
        let (mut homie, homie_handle) = homie_builder.spawn().await?;
        homie
            .add_node(Node {
                id: "left".to_owned(),
                name: "Left motor".to_owned(),
                node_type: "motor".to_owned(),
                properties: vec![
                    Property::integer("centre", "Centre", false, true, None, None),
                    Property::integer("target", "Target position", false, true, None, None),
                    Property::integer("position", "Actual position", false, true, None, None),
                ],
            })
            .await?;
        homie
            .add_node(Node {
                id: "right".to_owned(),
                name: "Right motor".to_owned(),
                node_type: "motor".to_owned(),
                properties: vec![
                    Property::integer("centre", "Centre", false, true, None, None),
                    Property::integer("target", "Target position", false, true, None, None),
                    Property::integer("position", "Actual position", false, true, None, None),
                ],
            })
            .await?;
        homie
            .add_node(Node {
                id: "general".to_owned(),
                name: "General settings".to_owned(),
                node_type: "general".to_owned(),
                properties: vec![
                    Property::integer(
                        "spring_constant",
                        "Spring constant",
                        false,
                        true,
                        None,
                        Some(SPRING_CONSTANT_STEP.into()..MAX_SPRING_CONSTANT.into()),
                    ),
                    Property::integer(
                        "min_speed",
                        "Min speed",
                        false,
                        true,
                        None,
                        Some((-MAX_MAX_SPEED).into()..0),
                    ),
                    Property::integer(
                        "max_speed",
                        "Max speed",
                        false,
                        true,
                        None,
                        Some(0..MAX_MAX_SPEED.into()),
                    ),
                    Property::float(
                        "scale",
                        "Scale",
                        false,
                        true,
                        None,
                        Some(1.0..MAX_SCALE.into()),
                    ),
                    Property::integer(
                        "battery_voltage",
                        "Battery voltage",
                        false,
                        true,
                        Some("mV"),
                        None,
                    ),
                    Property::integer(
                        "backup_battery_voltage",
                        "Backup battery voltage",
                        false,
                        true,
                        Some("mV"),
                        None,
                    ),
                    Property::integer("motor_current", "Motor current", false, true, None, None),
                    Property::boolean("charger_connected", "Charger connected", false, true, None),
                ],
            })
            .await?;
        homie.publish_value("left", "centre", 0).await?;
        homie.publish_value("right", "centre", 0).await?;
        homie
            .publish_value("general", "spring_constant", DEFAULT_SPRING_CONSTANT)
            .await?;
        homie
            .publish_value("general", "min_speed", DEFAULT_MAX_SPEED.negative)
            .await?;
        homie
            .publish_value("general", "max_speed", DEFAULT_MAX_SPEED.positive)
            .await?;
        homie
            .publish_value("general", "scale", DEFAULT_SCALE)
            .await?;
        homie.ready().await?;
        Ok(Self { homie, handle })
    }

    pub fn send_target(&self, side: Side, target: i64) {
        self.send_property(node_id(side), "target", target);
    }

    pub fn send_centre(&self, side: Side, centre: i64) {
        self.send_property(node_id(side), "centre", centre);
    }

    pub fn send_position(&self, side: Side, position: i64) {
        self.send_property(node_id(side), "position", position);
    }

    pub fn send_max_speed(&self, max_speed: SpeedLimits) {
        self.send_property("general", "max_speed", max_speed.positive);
        self.send_property("general", "min_speed", max_speed.negative);
    }

    pub fn send_spring_constant(&self, spring_constant: u16) {
        self.send_property("general", "spring_constant", spring_constant);
    }

    pub fn send_scale(&self, scale: f32) {
        self.send_property("general", "scale", scale);
    }

    pub fn send_battery_readings(
        &self,
        battery_voltage: u16,
        backup_battery_voltage: u16,
        motor_current: u16,
    ) {
        self.send_property("general", "battery_voltage", battery_voltage);
        self.send_property("general", "backup_battery_voltage", backup_battery_voltage);
        self.send_property("general", "motor_current", motor_current);
    }

    pub fn send_charge_state(&self, charger_connected: bool) {
        self.send_property("general", "charger_connected", charger_connected)
    }

    fn send_property(&self, node_id: &str, property_id: &str, value: impl ToString) {
        self.handle.block_on(async {
            if let Err(e) = self.homie.publish_value(node_id, property_id, value).await {
                error!("Error sending {} {} over MQTT: {}", node_id, property_id, e);
            }
        });
    }
}

fn node_id(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

/// Construct the `MqttOptions` for connecting to the MQTT broker based on configuration options or
/// defaults.
fn get_mqtt_options() -> MqttOptions {
    let mut mqtt_options = MqttOptions::new(HOMIE_DEVICE_ID, MQTT_HOST, MQTT_PORT);

    mqtt_options.set_keep_alive(5);
    if let (Some(username), Some(password)) = (MQTT_USERNAME, MQTT_PASSWORD) {
        mqtt_options.set_credentials(username, password);
    }

    if USE_TLS {
        let mut client_config = ClientConfig::new();
        client_config.root_store =
            rustls_native_certs::load_native_certs().expect("could not load platform certs");
        mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));
    }
    mqtt_options
}
