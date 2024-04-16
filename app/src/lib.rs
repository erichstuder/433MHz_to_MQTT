/*
* Everything in this library shall be runnable on any target, at least on:
* - thumbv6m-none-eabi
* - x86_64-unknown-linux-gnu
*/

#![cfg_attr(not(test), no_std)]


pub type SendMessage = fn(msg: &[u8]);
pub type EnterBootloaderCallback = fn();

pub struct Parser {
    enter_bootloader: EnterBootloaderCallback,
    send_message: SendMessage,
}

impl Parser {
    pub fn send_message(&self, msg: &[u8]) {
        (self.send_message)(msg);
    }

    pub fn parse_message(&self, msg: &[u8]) {
        if msg.eq(b"enter bootloader") {
            // class.write_packet(b"entering bootloader now\n").await?;
            // embassy_rp::rom_data::reset_to_usb_boot(0, 0);
            (self.send_message)(b"entering bootloader now\n");
            (self.enter_bootloader)();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send_message_mock(_msg: &[u8]) {
        println!("send_message_mock");
    }

    fn enter_bootloader_mock() {
        println!("Heeeeelllllo");
    }

    #[test]
    fn enter_bootloader_test() {
        let parser = Parser {
            send_message: send_message_mock,
            enter_bootloader: enter_bootloader_mock,
        };
        parser.parse_message(b"enter bootloader");
        assert_eq!(1,2);//let it fail
    }
}
