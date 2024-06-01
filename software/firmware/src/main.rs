#![no_std]
#![no_main]

mod usb_configuration;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_futures::join::join;
//use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::State;
use embassy_usb::driver::EndpointError;
use app;
use usb_configuration::UsbConfiguration;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut state = State::new();
    let mut usb_device = UsbConfiguration::new(p.USB, &mut state);

    let echo_fut = async {
        loop {
            usb_device.cdc_acm_class.wait_connection().await;
            let _ = echo(&mut usb_device.cdc_acm_class).await;
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_device.usb.run(), echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: embassy_rp::usb::Instance + 'd>(class: &mut embassy_usb::class::cdc_acm::CdcAcmClass<'d, embassy_rp::usb::Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    struct EnterBootloaderImpl;

    impl app::EnterBootloader for EnterBootloaderImpl {
        fn call(&mut self) {
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }
    }

    let mut parser = app::Parser::new(EnterBootloaderImpl);

    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        class.write_packet(b"echo: ").await?;
        class.write_packet(data).await?;
        class.write_packet(b"\n").await?;

        let answer = parser.parse_message(data);
        class.write_packet(answer).await?;
        class.write_packet(b"\n\n").await?;
    }
}
