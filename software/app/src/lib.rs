/*
* Everything in this library shall be runnable on any target, but at least on:
* - thumbv6m-none-eabi
* - x86_64-unknown-linux-gnu
*/

#![cfg_attr(not(test), no_std)]

mod parser;
pub use parser::{EnterBootloader, Parser};

mod buttons;
pub use buttons::Buttons;
