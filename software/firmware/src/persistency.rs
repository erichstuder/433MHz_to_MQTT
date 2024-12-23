//! This module handles the persistency.
//! It allows to persistently store and read data.

use embassy_rp::flash::{self, Flash};
use embassy_rp::peripherals::{FLASH, DMA_CH0};
use core::cmp::min;

struct Value {
    wifi_ssid: [u8; 32],
    wifi_password: [u8; 32],
    mqtt_host_ip: [u8; 32],
    mqtt_broker_username: [u8; 32],
    mqtt_broker_password: [u8; 32],
}

pub enum ValueId {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

// These values must align with the specifications in memory.x.
const FLASH_SIZE: usize = 2*1024*1024; // 2MB is valid for Raspberry Pi Pico.
const DATA_SIZE : usize = flash::ERASE_SIZE; // must be a multiple of ERASE_SIZE.
const DATA_ADDRESS_OFFSET: usize = FLASH_SIZE - flash::ERASE_SIZE; // put data at the end of flash memory.

pub struct Persistency {
    flash: Flash<'static, FLASH, flash::Async, FLASH_SIZE>,
    value: Value,
}

impl Persistency {
    pub fn new(flash: FLASH, dma: DMA_CH0) -> Self {
        let persistency = Self {
            flash: Flash::new(flash, dma),
            value: Value {
                wifi_ssid: [0; 32],
                wifi_password: [0; 32],
                mqtt_host_ip: [0; 32],
                mqtt_broker_username: [0; 32],
                mqtt_broker_password: [0; 32],
            },
        };
        persistency
    }

    fn read_all(&mut self) {
        let mut data: [u8; DATA_SIZE] = [0; DATA_SIZE];
        self.flash.blocking_read(DATA_ADDRESS_OFFSET as u32, &mut data).expect("Failed to read flash memory");

        self.value.wifi_ssid.copy_from_slice(&data[0..32]);
        self.value.wifi_password.copy_from_slice(&data[32..64]);
        self.value.mqtt_host_ip.copy_from_slice(&data[64..96]);
        self.value.mqtt_broker_username.copy_from_slice(&data[96..128]);
        self.value.mqtt_broker_password.copy_from_slice(&data[128..160]);
    }

    pub fn read(&mut self, value_id: ValueId) -> &[u8] {
        self.read_all();

        match value_id {
            ValueId::WifiSsid => &self.value.wifi_ssid,
            ValueId::WifiPassword => &self.value.wifi_password,
            ValueId::MqttHostIp => &self.value.mqtt_host_ip,
            ValueId::MqttBrokerUsername => &self.value.mqtt_broker_username,
            ValueId::MqttBrokerPassword => &self.value.mqtt_broker_password,
        }
    }

    pub fn store(&mut self, value: &[u8], value_id: ValueId) {
        fn copy_value_to_field(value: &[u8], target: &mut [u8]) {
            target.fill('\0' as u8);
            let copy_len = min(target.len(), value.len());
            target[..copy_len].copy_from_slice(&value[..copy_len]);
        }

        self.read_all();
        match value_id {
            ValueId::WifiSsid => copy_value_to_field(value, &mut self.value.wifi_ssid),
            ValueId::WifiPassword => copy_value_to_field(value, &mut self.value.wifi_password),
            ValueId::MqttHostIp => copy_value_to_field(value, &mut self.value.mqtt_host_ip),
            ValueId::MqttBrokerUsername => copy_value_to_field(value, &mut self.value.mqtt_broker_username),
            ValueId::MqttBrokerPassword => copy_value_to_field(value, &mut self.value.mqtt_broker_password),
        }

        let mut data: [u8; DATA_SIZE] = [0x00; DATA_SIZE];
        data[0..32].copy_from_slice(&self.value.wifi_ssid);
        data[32..64].copy_from_slice(&self.value.wifi_password);
        data[64..96].copy_from_slice(&self.value.mqtt_host_ip);
        data[96..128].copy_from_slice(&self.value.mqtt_broker_username);
        data[128..160].copy_from_slice(&self.value.mqtt_broker_password);

        self.flash.blocking_erase(DATA_ADDRESS_OFFSET as u32, (DATA_ADDRESS_OFFSET + DATA_SIZE) as u32).expect("Failed to erase flash memory");
        self.flash.blocking_write(DATA_ADDRESS_OFFSET as u32, &mut data).expect("Failed to write flash memory");
    }
}
