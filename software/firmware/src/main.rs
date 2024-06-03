#![no_std]
#![no_main]

mod usb_communication;
mod remote_receiver;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Pio, InterruptHandler};
use embassy_futures::join::join;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut remote_receiver = remote_receiver::RemoteReceiver::new(
        &mut common,
        sm0,
        p.PIN_28,
    );

    usb_communication::run(p.USB).await;

    //let (_, remote_read_result) = join(usb_communication::run(p.USB), remote_receiver.read()).await;
    //usb_communication::write(b"heeeeeeeeeeeeeeaaaaaaaaaaaaaaaaaaaaaaa").await;
}
