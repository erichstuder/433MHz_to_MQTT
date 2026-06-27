#![no_main]
#![cfg_attr(not(feature = "host-test"), no_std)]
#![deny(unsafe_code)]

use embassy_executor::{Spawner, main};
use embassy_rp as _;
use defmt_rtt as _;
use panic_probe as _;

#[main]
async fn main(_spawner: Spawner) {}
