#![no_std]
#![no_main]

mod usb_communication;
mod remote_receiver;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::pac::usb;
//use embassy_rp::pac::usb::Usb;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Pio, InterruptHandler};
use embassy_usb::class::cdc_acm::State;
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};

use usb_communication::UsbCommunication;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut remote_receiver = remote_receiver::RemoteReceiver::new(
        &mut common,
        sm0,
        p.PIN_28,
    );

    let mut state = State::new();
    let mut usb_communication = UsbCommunication::new(p.USB, &mut state);

    usb_communication.run().await;


    //let receiver_fut = async {
        loop {
            //usb_communication.usb.run().await;
            //usb_communication.cdc_acm_class.wait_connection().await;
            //let remote_read_result = remote_receiver.read().await;
            //usb_communication.write(remote_read_result).await;
            Timer::after(Duration::from_secs(2)).await;
        }
    //};

    //let (_, _) = join(usb_communication.run(), receiver_fut).await;
}
