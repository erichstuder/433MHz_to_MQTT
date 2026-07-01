//! Handles the communication via USB.

use defmt::unwrap;
use embassy_rp::{usb, Peri};
use embassy_executor::{Spawner, task};
use embassy_rp::peripherals::USB;
use embassy_rp::bind_interrupts;
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
use embassy_usb::driver::EndpointError as UsbEndpointError;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

type UsbDriver = usb::Driver<'static, USB>;

pub fn create(usb: Peri<'static, USB>, spawner: Spawner) -> (UsbSender, UsbReceiver) {
    const MAX_PACKET_SIZE: u8 = 64;

    bind_interrupts!(struct Irqs {
        USBCTRL_IRQ => usb::InterruptHandler<USB>;
    });

    let mut config = embassy_usb::Config::new(0x2E8A, 0x0005); //rpi pico w default vid=0x2E8A and pid=0x0005
    config.manufacturer = Some("github.com/erichstuder");
    config.product = Some("433MHz_to_MQTT");
    config.serial_number = Some("12345678");
    config.max_packet_size_0 = MAX_PACKET_SIZE;

    // TODO: these buffer sizes can most probably be reduced in size. find the minimum!
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let config_descriptor = CONFIG_DESCRIPTOR.init([0; _]);

    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let bos_descriptor = BOS_DESCRIPTOR.init([0; _]);

    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let msos_descriptor = MSOS_DESCRIPTOR.init([0; _]);

    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let control_buf = CONTROL_BUF.init([0; _]);

    let mut builder = embassy_usb::Builder::new(
        usb::Driver::new(usb, Irqs),
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    static CDC_ACM_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let cdc_acm_state = CDC_ACM_STATE.init(cdc_acm::State::new());
    let cdc_acm_class = CdcAcmClass::new(&mut builder, cdc_acm_state, MAX_PACKET_SIZE as u16);

    let (usb_sender, usb_receiver) = cdc_acm_class.split();

    static USB: StaticCell<UsbDevice<'static, UsbDriver>> = StaticCell::new();
    let usb = USB.init(builder.build());

    spawner.spawn(unwrap!(usb_task(usb)));

    ( UsbSender::new(usb_sender), UsbReceiver::new(usb_receiver) )
}

#[task]
async fn usb_task(usb: &'static mut UsbDevice<'static, UsbDriver>) -> ! {
    usb.run().await
}

type CdcAcmSender = cdc_acm::Sender<'static, usb::Driver<'static, USB>>;
type UsbSenderMutexed = Mutex<CriticalSectionRawMutex, CdcAcmSender>;

pub struct UsbSender {
    usb_sender: UsbSenderMutexed,
}

impl UsbSender {
    fn new(usb_sender: CdcAcmSender) -> Self {
        UsbSender {
            usb_sender: UsbSenderMutexed::new(usb_sender),
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), UsbEndpointError>{
        let mut usb_sender = self.usb_sender.lock().await;
        for chunk in data.chunks(usb_sender.max_packet_size() as usize) {
            usb_sender.write_packet(chunk).await?
        }
        Ok(())
    }
}

type CdcAcmReceiver = cdc_acm::Receiver<'static, usb::Driver<'static, USB>>;

pub struct UsbReceiver {
    usb_receiver: CdcAcmReceiver,
}

impl UsbReceiver {
    fn new(usb_receiver: CdcAcmReceiver) -> Self {
        UsbReceiver { usb_receiver }
    }

    pub async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, UsbEndpointError> {
        self.usb_receiver.wait_connection().await;
        self.usb_receiver.read_packet(buffer).await
    }
}


#[cfg(test)]
#[embedded_test::tests]
mod tests {
    use super::*;

    #[test]
    // Note: Not really a test. We just test if it can be created without error.
    async fn create(){
        let usb = embassy_rp::init(Default::default()).USB;
        let spawner = unsafe{ Spawner::for_current_executor() }.await;

        // Make sure the order of the outputs is as expected.
        let (_usb_sender, _usb_receiver): (UsbSender, UsbReceiver) = super::create(usb, spawner);

        // More is currently not tested as it mocking or switching between implementations would clutter the production code
        // and the use would be limited as there is no usb counterpart to talk to.
    }
}
