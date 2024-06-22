//use embassy_embedded_hal::flash;
use embassy_rp::flash::{Flash, Async, ERASE_SIZE};
use embassy_rp::peripherals::{FLASH, DMA_CH0};
use core::any::Any;
use core::mem::size_of;

struct Data {
    wifi_ssid: [u8; 32],
    wifi_password: [u8; 32],
    mqtt_host_ip: [u8; 32],
    mqtt_broker_username: [u8; 32],
    mqtt_broker_password: [u8; 32],
}

const DATA_SIZE: usize = size_of::<Data>();
const ADDR_OFFSET: u32 = 0x10000000;//0x100000;

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
        let mut data: [u8; DATA_SIZE] = [0x00; DATA_SIZE];
        //data[0..32].copy_from_slice(b"dumomyaaaaaaaaaaaaaaaaaaaaaaaaaa");

        // data[0..32].copy_from_slice(&self.data.wifi_ssid);
        // data[32..64].copy_from_slice(&self.data.wifi_password);
        // data[64..96].copy_from_slice(&self.data.mqtt_host_ip);
        // data[96..128].copy_from_slice(&self.data.mqtt_broker_username);
        // data[128..160].copy_from_slice(&self.data.mqtt_broker_password);

        let result = self.flash.blocking_erase(ADDR_OFFSET, ADDR_OFFSET + DATA_SIZE as u32);
        //let result = self.flash.blocking_write(0, &mut data);
        if let Err(e) = result{
            self.data.wifi_ssid.copy_from_slice(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            self.data.wifi_ssid[0] = e as u8;
            // self.data.wifi_ssid[5] = (capacity>>32) as u8;
            // self.data.wifi_ssid[6] = (capacity>>40) as u8;
            // self.data.wifi_ssid[7] = (capacity>>48) as u8;
            // self.data.wifi_ssid[8] = (capacity>>52) as u8;
        }
        else {
            self.data.wifi_ssid.copy_from_slice(b"zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        }
        //let _ = self.flash.blocking_write(ADDR_OFFSET, &mut data);
        // let capacity = self.flash.capacity();
        // self.data.wifi_ssid[1] = capacity as u8;
        // self.data.wifi_ssid[2] = (capacity>>8) as u8;
        // self.data.wifi_ssid[3] = (capacity>>16) as u8;
        // self.data.wifi_ssid[4] = (capacity>>24) as u8;
    }

    fn read(&mut self) {
        let mut data: [u8; 4096] = [0; 4096];
        let result = self.flash.blocking_read(0, &mut data);
        if let Err(e) = result{
            self.data.wifi_ssid.copy_from_slice(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            self.data.wifi_ssid[0] = e as u8;
            // self.data.wifi_ssid[5] = (capacity>>32) as u8;
            // self.data.wifi_ssid[6] = (capacity>>40) as u8;
            // self.data.wifi_ssid[7] = (capacity>>48) as u8;
            // self.data.wifi_ssid[8] = (capacity>>52) as u8;
        }
        else {
            self.data.wifi_ssid.copy_from_slice(&data[0..32]);
        }

        self.data.wifi_ssid.copy_from_slice(&data[0..32]);
        // self.data.wifi_password.copy_from_slice(&data[32..64]);
        // self.data.mqtt_host_ip.copy_from_slice(&data[64..96]);
        // self.data.mqtt_broker_username.copy_from_slice(&data[96..128]);
        // self.data.mqtt_broker_password.copy_from_slice(&data[128..160]);
    }

    pub fn store_wifi_ssid(&mut self, wifi_ssid: &[u8]) {
        self.data.wifi_ssid.copy_from_slice(b"dumomyaaaaaaaaaaaaaaaaaaaaaaaaaa");//wifi_ssid.try_into().unwrap();
        self.store();
    }

    pub fn read_wifi_ssid(&mut self) -> &[u8]{
        self.read();
        &self.data.wifi_ssid
    }
}
