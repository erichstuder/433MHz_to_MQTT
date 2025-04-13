use embassy_executor::task;

use app::parser::{self, Parser};
use crate::drivers::persistency::ParserToPersistency;

use crate::UsbSenderMutexed;
use crate::UsbReceiver;
use crate::PersistencyMutexed;

#[task]
pub async fn run(persistency: &'static PersistencyMutexed, mut usb_receiver: UsbReceiver, usb_sender: &'static UsbSenderMutexed) {
    struct EnterBootloader;
    impl parser::EnterBootloaderTrait for EnterBootloader {
        fn call(&mut self) {
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }
    }

    let parser_to_persistency = ParserToPersistency::new(persistency);
    let mut parser = Parser::new(EnterBootloader, parser_to_persistency);
    let mut buf = [0u8; 64];

    loop {
        usb_receiver.wait_connection().await;
        let byte_cnt = match usb_receiver.read_packet(&mut buf).await {
            Ok(byte_cnt) => byte_cnt,
            Err(_e) => {
                continue;
            }
        };
        let data = &buf[..byte_cnt];
        let mut answer = [0u8; 100];

        //TODO: remove code duplication
        match parser.parse_message(data, &mut answer).await {
            Ok(length) => {
                let mut sender = usb_sender.lock().await;
                sender.write_packet(&answer[..length]).await.unwrap();
                sender.write_packet("\n".as_bytes()).await.unwrap();
            },
            Err(e) => {
                let mut sender = usb_sender.lock().await;
                sender.write_packet(&"ERROR: ".as_bytes()).await.unwrap();
                sender.write_packet(&e.as_bytes()).await.unwrap();
                sender.write_packet("\n".as_bytes()).await.unwrap();
            },
        };
    }
}
