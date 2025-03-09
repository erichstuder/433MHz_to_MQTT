use embassy_executor::task;
use embassy_rp::peripherals::{FLASH, DMA_CH0};

use app::parser::{self, Parser};
use crate::drivers::persistency::ParserToPersistency;

use crate::UsbSenderMutex;
use crate::UsbReceiver;

#[task]
pub async fn run(flash: FLASH, dma_ch0: DMA_CH0, mut usb_receiver: UsbReceiver, usb_sender: &'static UsbSenderMutex) {
    struct EnterBootloader;
    impl parser::EnterBootloaderTrait for EnterBootloader {
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
        let answer = parser.parse_message(data);
        sender.write_packet(answer).await.unwrap();
    }
}
