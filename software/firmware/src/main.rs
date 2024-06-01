#![no_std]
#![no_main]

mod usb_communication;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
//use embassy_rp::usb::{Driver, Instance, InterruptHandler};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    usb_communication::run().await;
    //hier können andere futures verwendet werden z.B. mit futures::join!(usb_communication::run(), other_future);
}
