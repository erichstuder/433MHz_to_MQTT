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
    let mut bytes = [0u8; 64]; //TODO: this should be the max packet size, see usb_communication.rs
    let mut receive_buffer = [0u8; 128];
    let mut receive_buffer_index = 0usize;
    let mut ignore_message = false;

    loop {
        usb_receiver.wait_connection().await;
        let byte_cnt = match usb_receiver.read_packet(&mut bytes).await {
            Ok(byte_cnt) => byte_cnt,
            Err(_e) => continue,
        };

        for n in 0..byte_cnt {
            if bytes[n] == b'\n' {
                if ignore_message {
                    ignore_message = false;
                }
                else {
                    let mut answer = [0u8; 100];
                    //TODO: remove code duplication
                    match parser.parse_message(&receive_buffer[..receive_buffer_index], &mut answer).await {
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
                receive_buffer_index = 0;
            }
            else {
                if receive_buffer_index < receive_buffer.len() {
                    receive_buffer[receive_buffer_index] = bytes[n];
                    receive_buffer_index += 1;
                } else {
                    ignore_message = true;
                    let mut sender = usb_sender.lock().await;
                    sender.write_packet("receive buffer overflow, this message is ignored: ".as_bytes()).await.unwrap();
                    for chunk in receive_buffer.chunks(sender.max_packet_size() as usize) {
                        sender.write_packet(chunk).await.unwrap();
                    }
                    sender.write_packet("...\n".as_bytes()).await.unwrap();
                    receive_buffer_index = 0;
                }
            }
        }
    }
}
