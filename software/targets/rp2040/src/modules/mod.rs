#[cfg(not(feature = "host-test"))]
pub mod usb_communication;

pub mod flash_persistency;
pub mod remote_receiver;
pub mod mqtt;

// During tests some interrupt bindings are needed.
// As they are handled globally, binding them in the test modules would lead to binding conflicts.
// Therefore we bind them here, and the test modules can use them.
#[cfg(feature = "target-test")]
mod test_setup {
    use embassy_rp::{
        bind_interrupts,
        pio,
        dma,
        peripherals::{PIO1, DMA_CH0, DMA_CH1},
    };

    bind_interrupts!(pub(crate) struct Pio1Irqs {
        PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    });

    bind_interrupts!(pub(crate) struct DmaIrqs {
        DMA_IRQ_0 =>
            dma::InterruptHandler<DMA_CH0>,
            dma::InterruptHandler<DMA_CH1>;
    });
}
