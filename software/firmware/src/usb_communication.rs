use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance};
use embassy_usb::{Builder, Config, UsbDevice};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use app;
use embassy_futures::join::join;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

pub struct UsbCommunication<'a> {
    pub cdc_acm_class: CdcAcmClass<'a, Driver<'a, USB>>,
    pub usb: UsbDevice<'a, Driver<'a, USB>>,
}

impl<'a> UsbCommunication<'a> {
    pub fn new(usb: USB, state: &'a mut State<'a>) -> Self{
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

        static mut DEVICE_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut CONFIG_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut BOS_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut CONTROL_BUF: [u8; 64] = [0; 64];

        #[allow(unknown_lints)] //TODO: all this allow stuff should be removed
        let mut builder = Builder::new(
            Driver::new(usb, Irqs),
            config,
            #[allow(static_mut_refs)]
            unsafe{ &mut DEVICE_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut CONFIG_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut BOS_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut CONTROL_BUF },
        );

        let cdc_acm_class = CdcAcmClass::new(&mut builder, state, 64);
        let usb = builder.build();

        Self {
            cdc_acm_class,
            usb,
        }
    }

    pub async fn run(&mut self) {
        let cdc_acm_class = &mut self.cdc_acm_class;
        let echo_fut = async {
            loop {
                cdc_acm_class.wait_connection().await;
                let _ = echo(cdc_acm_class).await;
            }
        };
        join(self.usb.run(), echo_fut).await;
    }

    pub async fn write(&mut self, data: &[u8]) {
        self.cdc_acm_class.write_packet(data).await.unwrap();
    }
}


async fn echo<'d, T: Instance + 'd>(cdc_acm_class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    struct EnterBootloaderImpl;

    impl app::EnterBootloader for EnterBootloaderImpl {
        fn call(&mut self) {
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }
    }

    let mut parser = app::Parser::new(EnterBootloaderImpl);

    loop {
        let n = cdc_acm_class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        cdc_acm_class.write_packet(b"echo: ").await?;
        cdc_acm_class.write_packet(data).await?;
        cdc_acm_class.write_packet(b"\n").await?;

        let answer = parser.parse_message(data);
        cdc_acm_class.write_packet(answer).await?;
        cdc_acm_class.write_packet(b"\n\n").await?;
    }
}
