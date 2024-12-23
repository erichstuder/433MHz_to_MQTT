//! This library contains the hardware independent part of the the code.
//! At the moment there is no simple possibility to unit-test no_std rust code.
//! Testing is then possible in a usual way.
//!
//! Everything in this library shall be runnable on any target, but at least on:
//!
//! - thumbv6m-none-eabi
//! - x86_64-unknown-linux-gnu


#![cfg_attr(not(test), no_std)]

pub mod parser;
pub mod buttons;
