use core::panic;

use embassy_executor::task;

use app::parser::{self, Parser};
use crate::drivers::persistency::ParserToPersistency;

use crate::drivers::usb_communication;
use crate::UsbSenderMutexed;
use crate::UsbReceiver;

use crate::PersistencyMutexed;
use embassy_usb::driver::EndpointError;

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
    let mut bytes = [0u8; usb_communication::MAX_PACKET_SIZE as usize];
    let mut receive_buffer = [0u8; 128];
    let mut receive_buffer_index = 0usize;
    let mut ignore_message = false;

    loop {
        usb_receiver.wait_connection().await;
        let byte_cnt = match usb_receiver.read_packet(&mut bytes).await {
            Ok(byte_cnt) => byte_cnt,
            Err(e) => {
                match e {
                    EndpointError::BufferOverflow => {
                        let mut sender = usb_sender.lock().await;
                        sender.write_packet("receive buffer overflow, this message is ignored: ".as_bytes()).await.unwrap();
                        for chunk in bytes.chunks(sender.max_packet_size() as usize) {
                            sender.write_packet(chunk).await.unwrap();
                        }
                        sender.write_packet("... The system now shuts down. Goodbye.\n".as_bytes()).await.unwrap();
                        // This should never ever happen. So a panic is appropriate.
                        panic!("receive buffer overflow");
                    },
                    EndpointError::Disabled => {
                        // This is expected when the USB cable is disconnected.
                        continue;
                    },
                }
            },
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
