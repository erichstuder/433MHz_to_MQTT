use embassy_executor::task;
use embassy_rp::peripherals::{FLASH, DMA_CH0};

use app::parser::{self, Parser};
use crate::drivers::persistency::{self, Persistency};
use crate::drivers::usb_communication;

use crate::UsbSenderMutex;
use crate::UsbReceiver;

struct ParserToPersistency {
    persistency: Persistency,
}
impl ParserToPersistency {
    fn new(flash: FLASH, dma_ch0: DMA_CH0) -> Self {
        Self {
            persistency: Persistency::new(flash, dma_ch0),
        }
    }
}
impl parser::Persistency for ParserToPersistency {
    fn store(&mut self, value: &[u8], value_id: parser::ValueId) {
        match value_id {
            parser::ValueId::WifiSsid           => self.persistency.store(value, persistency::ValueId::WifiSsid),
            parser::ValueId::WifiPassword       => self.persistency.store(value, persistency::ValueId::WifiPassword),
            parser::ValueId::MqttHostIp         => self.persistency.store(value, persistency::ValueId::MqttHostIp),
            parser::ValueId::MqttBrokerUsername => self.persistency.store(value, persistency::ValueId::MqttBrokerUsername),
            parser::ValueId::MqttBrokerPassword => self.persistency.store(value, persistency::ValueId::MqttBrokerPassword),
        }
    }

    fn read(&mut self, value_id: parser::ValueId) -> &[u8] {
        match value_id {
            parser::ValueId::WifiSsid           => self.persistency.read(persistency::ValueId::WifiSsid),
            parser::ValueId::WifiPassword       => self.persistency.read(persistency::ValueId::WifiPassword),
            parser::ValueId::MqttHostIp         => self.persistency.read(persistency::ValueId::MqttHostIp),
            parser::ValueId::MqttBrokerUsername => self.persistency.read(persistency::ValueId::MqttBrokerUsername),
            parser::ValueId::MqttBrokerPassword => self.persistency.read(persistency::ValueId::MqttBrokerPassword),
        }
    }
}



#[task]
pub async fn run(flash: FLASH, dma_ch0: DMA_CH0, mut usb_receiver: UsbReceiver, usb_sender: &'static UsbSenderMutex) {
    struct EnterBootloader;
    impl parser::EnterBootloader for EnterBootloader {
        fn call(&mut self) {
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }
    }

    let parser_to_persistency = ParserToPersistency::new(flash, dma_ch0);
    let mut parser = Parser::new(EnterBootloader, parser_to_persistency);
    let mut buf: [u8; 64] = [0; 64];

    loop {
        usb_receiver.wait_connection().await;
        let byte_cnt = match usb_receiver.read_packet(&mut buf).await {
            Ok(byte_cnt) => byte_cnt,
            Err(_e) => {
                continue;
            }
        };
        let data = &buf[..byte_cnt];
        let mut sender = usb_sender.lock().await;
        let _ = usb_communication::parse_message(data, &mut sender, &mut parser).await;
    }
}
