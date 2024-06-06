#![no_std]
#![no_main]

mod usb_communication;
mod remote_receiver;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Pio, InterruptHandler};
use embassy_usb::class::cdc_acm::State;
use embassy_futures::join::join;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::class::cdc_acm::Sender;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;

use usb_communication::UsbCommunication;

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

    let mut state = State::new();
    let usb_communication = UsbCommunication::new(p.USB, &mut state);

    let mut usb = usb_communication.usb;
    let (sender, mut receiver) = usb_communication.cdc_acm_class.split();
    let sender: Mutex<CriticalSectionRawMutex, Sender<Driver<USB>>> = Mutex::new(sender);

    let receiver_fut = async {
        loop {
            let remote_read_result = remote_receiver.read().await;
            {
                let mut sender = sender.lock().await;
                let _ = sender.write_packet(&remote_read_result.to_ne_bytes()).await;
                let _ = sender.write_packet(b"\n").await;
            }
        }
    };

    let echo_fut = async {
        let mut buf = [0; 64];
        loop {
            receiver.wait_connection().await;
            let n = match receiver.read_packet(&mut buf).await {
                Ok(n) => n,
                Err(_e) => {
                    // Handle the error
                    continue;
                }
            };
            let data = &buf[..n];
            {
                let mut sender = sender.lock().await;
                let _ = usb_communication::echo(data, &mut sender).await;
            }
        }
    };
    join(usb.run(), join(echo_fut, receiver_fut)).await;
}
