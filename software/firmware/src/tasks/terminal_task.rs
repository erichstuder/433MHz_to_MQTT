//! Handling of the terminal communication.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        use core::panic;
        use embassy_executor::task;
        use crate::drivers::parser::Parser;
        use crate::drivers::persistency::{Persistency, ParserToPersistency};
        use crate::drivers::usb_communication::{self, UsbReceiver, UsbSender};
        use embassy_usb::driver::EndpointError;
    }
}

#[cfg(not(test))]
#[task]
pub async fn run(persistency: &'static Persistency, mut usb_receiver: UsbReceiver, usb_sender: &'static UsbSender) {
    let parser_to_persistency = ParserToPersistency::new(persistency);
    let mut parser = Parser::new(parser_to_persistency);
    let mut bytes = [0u8; usb_communication::MAX_PACKET_SIZE as usize];
    let mut receive_buffer = [0u8; 128];
    let mut receive_buffer_index = 0usize;
    let mut ignore_message = false;

    loop {
        let byte_cnt = match usb_receiver.read_packet(&mut bytes).await {
            Ok(byte_cnt) => byte_cnt,
            Err(e) => {
                match e {
                    EndpointError::BufferOverflow => {
                        usb_sender.send("receive buffer overflow, this message is ignored: ".as_bytes()).await.unwrap();
                        usb_sender.send(&bytes).await.unwrap();
                        usb_sender.send("... The system now shuts down. Goodbye.\n".as_bytes()).await.unwrap();
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
                    match parser.parse_message(&receive_buffer[..receive_buffer_index], &mut answer).await {
                        Ok(length) => {
                            usb_sender.send(&answer[..length]).await.unwrap();
                        },
                        Err(e) => {
                            usb_sender.send(&"ERROR: ".as_bytes()).await.unwrap();
                            usb_sender.send(&e.as_bytes()).await.unwrap();
                        },
                    };
                    usb_sender.send("\n".as_bytes()).await.unwrap();
                }
                receive_buffer_index = 0;
            }
            else {
                if receive_buffer_index < receive_buffer.len() {
                    receive_buffer[receive_buffer_index] = bytes[n];
                    receive_buffer_index += 1;
                } else {
                    ignore_message = true;
                    usb_sender.send("receive buffer overflow, this message is ignored: ".as_bytes()).await.unwrap();
                    usb_sender.send("...\n".as_bytes()).await.unwrap();
                    receive_buffer_index = 0;
                }
            }
        }
    }
}
