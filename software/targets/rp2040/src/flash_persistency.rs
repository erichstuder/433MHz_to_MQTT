use embassy_rp::Peripherals;

use core::ops::Range;

use lib::persistency::Persistency;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{self, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_rp::peripherals::DMA_CH0;

// use core::panic::PanicInfo;
// use embassy_rp::Peri;

// use embedded_test;

// TODO: it should not have to align with memory.x but be defined in one place.
// These values must align with the specifications in memory.x.
const FLASH_SIZE: usize = 2*1024*1024; // 2MB is valid for Raspberry Pi Pico.
const ADDRESS_RANGE: Range<u32> = (FLASH_SIZE as u32 - 2*flash::ERASE_SIZE as u32)..FLASH_SIZE as u32;

bind_interrupts!(struct DmaIrqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

pub type FlashPersistency = Persistency<Flash<'static, FLASH, flash::Async, FLASH_SIZE>>;

pub fn init(peripherals: Peripherals) -> FlashPersistency {
    let flash = Flash::new(peripherals.FLASH, peripherals.DMA_CH0, DmaIrqs);
    Persistency::new(flash, ADDRESS_RANGE)
}

#[cfg(test)]
#[embedded_test::tests]
mod tests {
    use super::*;
    use defmt_rtt as _;
    use defmt::assert_eq;

    #[init]
    fn init() -> Peripherals {
        embassy_rp::init(Default::default())
    }

    #[test]
    fn dummy_test(p: Peripherals) {
        let _flash_persistency = super::init(p);
        assert_eq!(1, 1);
    }

    #[test]
    fn check_the_erase_size() {
        assert_eq!(flash::ERASE_SIZE, 4096);
    }
}
