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
        let mut answer: [u8; 32] = ['\0' as u8; 32];
        parser.parse_message(data, &mut answer).await;
        let len = answer.iter().position(|&x| x == b'\0').unwrap_or(answer.len());

        let mut sender = usb_sender.lock().await;

        sender.write_packet(&answer[..len]).await.unwrap();
        sender.write_packet("\n".as_bytes()).await.unwrap();
    }
}
