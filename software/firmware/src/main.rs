#![no_std]
#![no_main]

//use defmt::{info, panic};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::{Builder, Config};
use embassy_usb::class::cdc_acm::State;
//use embassy_executor::Spawner;
use embassy_futures::join::join;
//use embassy_rp::usb::{Driver, Instance, InterruptHandler};
//use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use app;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});


struct UsbDevice<'a> {
    driver: Driver<'a, USB>,
    config: Config<'a>,
    device_descriptor: [u8; 256],
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    control_buf: [u8; 64],
}

impl<'a> UsbDevice<'a> {
    fn new(usb: USB) -> Self {
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("github.com/erichstuder");
        config.product = Some("433MHz_to_MQTT");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        Self {
            driver: Driver::new(usb, Irqs),
            config,
            device_descriptor: [0; 256],
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            control_buf: [0; 64],
        }
    }
}


#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut usb_device = UsbDevice::new(p.USB);

    let mut state = State::new();

    let mut builder = Builder::new(
        usb_device.driver,
        usb_device.config,
        &mut usb_device.device_descriptor,
        &mut usb_device.config_descriptor,
        &mut usb_device.bos_descriptor,
        &mut [], // no msos descriptors
        &mut usb_device.control_buf,
    );

    //Create classes on the builder.
    let mut cdc_acm_class = embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            cdc_acm_class.wait_connection().await;
            //info!("Connected");
            let _ = echo(&mut cdc_acm_class).await;
            //info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
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
