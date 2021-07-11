use crate::config::{get_mqtt_options, MqttConfig};
use crate::controller::{
    DEFAULT_MAX_SPEED, DEFAULT_SCALE, DEFAULT_SPRING_CONSTANT, MAX_MAX_SPEED, MAX_SCALE,
    MAX_SPRING_CONSTANT, SPRING_CONSTANT_STEP,
};
use eyre::Report;
use homie_device::{HomieDevice, Node, Property};
use log::{error, trace};
use messages::{Side, SpeedLimits};
use tokio::runtime::Handle;

const HOMIE_PREFIX: &str = "homie";
const HOMIE_DEVICE_ID: &str = "hoverkite";
const HOMIE_DEVICE_NAME: &str = "Hoverkite";

pub struct Homie<'a> {
    homie: Option<HomieDevice>,
    handle: &'a Handle,
}

impl<'a> Homie<'a> {
    pub fn connect_and_start(
        handle: &'a Handle,
        config: Option<MqttConfig>,
    ) -> Result<Homie<'a>, Report> {
        let homie = if let Some(config) = config {
            Some(handle.block_on(make_homie_device(config))?)
        } else {
            None
        };
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
        side: Side,
        battery_voltage: u16,
        backup_battery_voltage: u16,
        motor_current: u16,
    ) {
        let node_id = node_id(side);
        self.send_property(node_id, "battery_voltage", battery_voltage);
        self.send_property(node_id, "backup_battery_voltage", backup_battery_voltage);
        self.send_property(node_id, "motor_current", motor_current);
    }

    pub fn send_charge_state(&self, side: Side, charger_connected: bool) {
        self.send_property(node_id(side), "charger_connected", charger_connected)
    }

    fn send_property(&self, node_id: &str, property_id: &str, value: impl ToString) {
        if let Some(homie) = &self.homie {
            self.handle.block_on(async {
                if let Err(e) = homie.publish_value(node_id, property_id, value).await {
                    error!("Error sending {} {} over MQTT: {}", node_id, property_id, e);
                }
            });
        } else {
            trace!(
                "Would publish {}/{} = {}",
                node_id,
                property_id,
                value.to_string()
            );
        }
    }
}

async fn make_homie_device(config: MqttConfig) -> Result<HomieDevice, Report> {
    let mqtt_options = get_mqtt_options(config, HOMIE_DEVICE_ID);
    let device_base = format!("{}/{}", HOMIE_PREFIX, HOMIE_DEVICE_ID);
    let homie_builder = HomieDevice::builder(&device_base, HOMIE_DEVICE_NAME, mqtt_options);
    let (mut homie, homie_handle) = homie_builder.spawn().await?;
    let motor_properties = vec![
        Property::integer("centre", "Centre", false, true, None, None),
        Property::integer("target", "Target position", false, true, None, None),
        Property::integer("position", "Actual position", false, true, None, None),
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
    ];
    homie
        .add_node(Node {
            id: "left".to_owned(),
            name: "Left motor".to_owned(),
            node_type: "motor".to_owned(),
            properties: motor_properties.clone(),
        })
        .await?;
    homie
        .add_node(Node {
            id: "right".to_owned(),
            name: "Right motor".to_owned(),
            node_type: "motor".to_owned(),
            properties: motor_properties,
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
    Ok(homie)
}

fn node_id(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}
