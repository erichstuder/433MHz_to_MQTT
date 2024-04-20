/*
* Everything in this library shall be runnable on any target, but at least on:
* - thumbv6m-none-eabi
* - x86_64-unknown-linux-gnu
*/

#![cfg_attr(not(test), no_std)]

extern crate alloc;
use alloc::boxed::Box;

pub struct Parser {
    send_message: Box<dyn FnMut(&[u8])>,
    enter_bootloader: Box<dyn FnMut()>,
}

impl Parser {
    pub fn parse_message(&mut self, msg: &[u8]) {
        if msg.eq(b"enter bootloader") {
            (self.send_message)(b"entering bootloader now\n");
            (self.enter_bootloader)();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{Ordering, AtomicBool};

    #[test]
    // TODO: This is not a great test. But as I'm still learning, I leave it like this for now.
    fn test_enter_bootloader() {
        fn mock_send_message(_msg: &[u8]) {}
        //static mut RESULT_MSG: &[u8];
        static ENTER_BOOTLOADER_CALLED: AtomicBool = AtomicBool::new(false);

        let mut parser = Parser {
            send_message: Box::new(mock_send_message),
            //send_message: Box::new(|msg: &[u8]| RESULT_MSG = msg),
            enter_bootloader: Box::new(|| ENTER_BOOTLOADER_CALLED.store(true, Ordering::Relaxed)),
        };

        parser.parse_message(b"enter bootloader");

        assert!(ENTER_BOOTLOADER_CALLED.load(Ordering::Relaxed));
    }
}
