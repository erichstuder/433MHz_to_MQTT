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

const ADDR_OFFSET: u32 = 0x100000;

pub struct Persistency {
    flash: Flash<'static, FLASH, Async, {2*1024*1024}>,
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
        let mut data: [u8; ERASE_SIZE] = [0x00; ERASE_SIZE];

        data[0..32].copy_from_slice(&self.data.wifi_ssid);
        data[32..64].copy_from_slice(&self.data.wifi_password);
        data[64..96].copy_from_slice(&self.data.mqtt_host_ip);
        data[96..128].copy_from_slice(&self.data.mqtt_broker_username);
        data[128..160].copy_from_slice(&self.data.mqtt_broker_password);

        let result1 = self.flash.blocking_erase(ADDR_OFFSET, ADDR_OFFSET + ERASE_SIZE as u32);
        let result3 = self.flash.blocking_write(ADDR_OFFSET, &mut data);
    }

    fn read(&mut self) {
        let mut data: [u8; ERASE_SIZE] = [0; ERASE_SIZE];
        let result = self.flash.blocking_read(ADDR_OFFSET, &mut data);

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

    pub fn read_wifi_ssid(&mut self) -> &[u8]{
        self.read();
        &self.data.wifi_ssid
    }
}
