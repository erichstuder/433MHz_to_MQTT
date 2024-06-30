//use embassy_embedded_hal::flash;
use embassy_rp::flash::{Flash, Async, ERASE_SIZE};
use embassy_rp::peripherals::{FLASH, DMA_CH0};
use core::cmp::min;

struct Data {
    wifi_ssid: [u8; 32],
    wifi_password: [u8; 32],
    mqtt_host_ip: [u8; 32],
    mqtt_broker_username: [u8; 32],
    mqtt_broker_password: [u8; 32],
}

// These values must align with the specifications in memory.x.
const FLASH_SIZE: usize = 2*1024*1024; // 2MB is valid for Raspberry Pi Pico.
const DATA_SIZE : usize = ERASE_SIZE; // must be a multiple of ERASE_SIZE.
const DATA_ADDRESS_OFFSET: usize = FLASH_SIZE - ERASE_SIZE; // put data at the end of flash memory.

pub struct Persistency {
    flash: Flash<'static, FLASH, Async, FLASH_SIZE>,
    data: Data,
}

impl Persistency {
    pub fn new(flash: FLASH, dma: DMA_CH0) -> Self {
        let mut persistency = Self {
            flash: Flash::new(flash, dma),
            data: Data {
                wifi_ssid: [0; 32],
                wifi_password: [0; 32],
                mqtt_host_ip: [0; 32],
                mqtt_broker_username: [0; 32],
                mqtt_broker_password: [0; 32],
            },
        };
        persistency.read();
        persistency
    }

    fn store(&mut self) {
        let mut data: [u8; DATA_SIZE] = [0x00; DATA_SIZE];

        data[0..32].copy_from_slice(&self.data.wifi_ssid);
        data[32..64].copy_from_slice(&self.data.wifi_password);
        data[64..96].copy_from_slice(&self.data.mqtt_host_ip);
        data[96..128].copy_from_slice(&self.data.mqtt_broker_username);
        data[128..160].copy_from_slice(&self.data.mqtt_broker_password);

        self.flash.blocking_erase(DATA_ADDRESS_OFFSET as u32, (DATA_ADDRESS_OFFSET + DATA_SIZE) as u32).expect("Failed to erase flash memory");
        self.flash.blocking_write(DATA_ADDRESS_OFFSET as u32, &mut data).expect("Failed to write flash memory");
    }

    fn read(&mut self) {
        let mut data: [u8; DATA_SIZE] = [0; DATA_SIZE];
        self.flash.blocking_read(DATA_ADDRESS_OFFSET as u32, &mut data).expect("Failed to read flash memory");

        self.data.wifi_ssid.copy_from_slice(&data[0..32]);
        self.data.wifi_password.copy_from_slice(&data[32..64]);
        self.data.mqtt_host_ip.copy_from_slice(&data[64..96]);
        self.data.mqtt_broker_username.copy_from_slice(&data[96..128]);
        self.data.mqtt_broker_password.copy_from_slice(&data[128..160]);
    }

    pub fn store_wifi_ssid(&mut self, wifi_ssid: &[u8]) {
        self.data.wifi_ssid.fill('\0' as u8);

        let copy_len = min(wifi_ssid.len(), self.data.wifi_ssid.len());
        self.data.wifi_ssid[..copy_len].copy_from_slice(wifi_ssid[..copy_len].as_ref());
        self.store();
    }

    pub fn store_wifi_password(&mut self, wifi_password: &[u8]) {
        self.data.wifi_password.fill('\0' as u8);

        let copy_len = min(wifi_password.len(), self.data.wifi_password.len());
        self.data.wifi_password[..copy_len].copy_from_slice(wifi_password[..copy_len].as_ref());
        self.store();
    }

    pub fn store_mqtt_host_ip(&mut self, mqtt_host_ip: &[u8]) {
        self.data.mqtt_host_ip.fill('\0' as u8);

        let copy_len = min(mqtt_host_ip.len(), self.data.mqtt_host_ip.len());
        self.data.mqtt_host_ip[..copy_len].copy_from_slice(mqtt_host_ip[..copy_len].as_ref());
        self.store();
    }

    pub fn store_mqtt_broker_username(&mut self, mqtt_broker_username: &[u8]) {
        self.data.mqtt_broker_username.fill('\0' as u8);

        let copy_len = min(mqtt_broker_username.len(), self.data.mqtt_broker_username.len());
        self.data.mqtt_broker_username[..copy_len].copy_from_slice(mqtt_broker_username[..copy_len].as_ref());
        self.store();
    }

    pub fn store_mqtt_broker_password(&mut self, mqtt_broker_password: &[u8]) {
        self.data.mqtt_broker_password.fill('\0' as u8);

        let copy_len = min(mqtt_broker_password.len(), self.data.mqtt_broker_password.len());
        self.data.mqtt_broker_password[..copy_len].copy_from_slice(mqtt_broker_password[..copy_len].as_ref());
        self.store();
    }

    pub fn read_wifi_ssid(&mut self) -> &[u8]{
        self.read();
        &self.data.wifi_ssid
    }

    pub fn read_wifi_password(&mut self) -> &[u8]{
        self.read();
        &self.data.wifi_password
    }

    pub fn read_mqtt_host_ip(&mut self) -> &[u8]{
        self.read();
        &self.data.mqtt_host_ip
    }

    pub fn read_mqtt_broker_username(&mut self) -> &[u8]{
        self.read();
        &self.data.mqtt_broker_username
    }

    pub fn read_mqtt_broker_password(&mut self) -> &[u8]{
        self.read();
        &self.data.mqtt_broker_password
    }
}
