#![no_std]
#![no_main]
#![deny(unsafe_code)]

// use embassy_executor::{Spawner, main};

use embassy_executor::Spawner;
#[cfg(not(test))]
use embassy_executor::main;

use embassy_rp as _;
use defmt_rtt as _;
use panic_probe as _;

// mod store;

#[main]
async fn main(_spawner: Spawner) {}
