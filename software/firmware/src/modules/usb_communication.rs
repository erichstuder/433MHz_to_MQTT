//! Handles the communication via USB.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_executor::{Spawner, task};
        use embassy_usb::UsbDevice;
        use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
        use embassy_usb::driver::EndpointError;
        use embassy_sync::mutex::Mutex;
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

        use static_cell::StaticCell;

        type UsbDriver = usb::Driver<'static, USB>;

        pub const MAX_PACKET_SIZE: u8 = 64;
    }
}

use embassy_rp::peripherals::USB;
use embassy_rp::usb;



use embassy_usb::driver::EndpointError as UsbEndpointError;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

pub struct UsbDisconnected {}

impl From<UsbEndpointError> for UsbDisconnected {
    fn from(val: UsbEndpointError) -> Self {
        match val {
            UsbEndpointError::BufferOverflow => panic!("Buffer overflow"),
            UsbEndpointError::Disabled => UsbDisconnected {},
        }
    }
}

#[cfg(not(test))]
pub fn create(usb: USB, spawner: Spawner) -> (UsbReceiver, UsbSender) {
    let mut config = embassy_usb::Config::new(0x2E8A, 0x0005); //rpi pico w default vid=0x2E8A and pid=0x0005
    config.manufacturer = Some("github.com/erichstuder");
    config.product = Some("433MHz_to_MQTT");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = MAX_PACKET_SIZE;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 256]);

    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 256]);

    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let msos_descriptor = MSOS_DESCRIPTOR.init([0; 256]);

    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let control_buf = CONTROL_BUF.init([0; 64]);

    let mut builder = embassy_usb::Builder::new(
        usb::Driver::new(usb, Irqs),
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor, //TODO: Embassy does not have this in every example. Is it necessary?
        control_buf,
    );

    static CDC_ACM_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let cdc_acm_state = CDC_ACM_STATE.init(cdc_acm::State::new());
    let cdc_acm_class = CdcAcmClass::new(&mut builder, cdc_acm_state, MAX_PACKET_SIZE as u16);

    let (usb_sender, usb_receiver) = cdc_acm_class.split();
    //let usb_sender_mutexed = UsbSenderMutexed::new(usb_sender);

    static USB: StaticCell<UsbDevice<'static, UsbDriver>> = StaticCell::new();
    let usb = USB.init(builder.build());

    spawner.spawn(usb_task(usb)).unwrap();

    ( UsbReceiver::new(usb_receiver), UsbSender::new(usb_sender) )
}

#[cfg(not(test))]
#[task]
async fn usb_task(usb: &'static mut UsbDevice<'static, UsbDriver>) -> ! {
    usb.run().await
}

#[cfg(not(test))]
pub struct UsbReceiver {
    usb_receiver: cdc_acm::Receiver<'static, usb::Driver<'static, USB>>,
}

#[cfg(not(test))]
impl UsbReceiver {
    fn new(usb_receiver: cdc_acm::Receiver<'static, usb::Driver<'static, USB>>) -> Self {
        UsbReceiver { usb_receiver }
    }

    pub async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, EndpointError> {
        self.usb_receiver.wait_connection().await;
        self.usb_receiver.read_packet(buffer).await
    }
}

#[cfg(not(test))]
type UsbSenderMutexed = Mutex<CriticalSectionRawMutex, cdc_acm::Sender<'static, usb::Driver<'static, USB>>>;

#[cfg(not(test))]
pub struct UsbSender {
    usb_sender: UsbSenderMutexed,
}

#[cfg(not(test))]
impl UsbSender {
    fn new(usb_sender: cdc_acm::Sender<'static, usb::Driver<'static, USB>>) -> Self {
        UsbSender {
            usb_sender: UsbSenderMutexed::new(usb_sender),
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), EndpointError>{
        let mut usb_sender = self.usb_sender.lock().await;
        for chunk in data.chunks(usb_sender.max_packet_size() as usize) {
            usb_sender.write_packet(chunk).await?
        }
        Ok(())
    }
}
