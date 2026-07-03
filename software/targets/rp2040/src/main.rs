#![no_main]
#![cfg_attr(not(feature = "host-test"), no_std)]
#![deny(unsafe_code)]

use embassy_executor::{Spawner, main};
use embassy_rp as _;
use defmt_rtt as _;
use panic_probe as _;

use lib::parser::{self, Parser};
use firmware::modules::{
    usb_communication,
    flash_persistency,
};

struct EnterBootloader;
impl parser::Command for EnterBootloader {
    fn cmd_str(&self) -> &'static [u8] {
        b"enter bootloader"
    }

    fn run(&self, answer: &mut [u8]) -> Result<usize, &'static str> {
        embassy_rp::rom_data::reset_to_usb_boot(0, 0);

        // Note: Probably this message won't be seen, because of immediate restart.
        let text = b"entering bootloader now";
        answer.copy_from_slice(text);
        Ok(text.len())
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let (_usb_sender, _usb_receiver) = usb_communication::create(peripherals.USB, spawner);
    let mut persistency = flash_persistency::init(peripherals.FLASH, peripherals.DMA_CH0);
    let additional_command = EnterBootloader;

    let _parser = Parser::new(&mut persistency, additional_command);
}
