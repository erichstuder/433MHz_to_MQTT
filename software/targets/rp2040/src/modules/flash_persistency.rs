use core::ops::Range;

use lib::persistency::{self, Persistency};
pub use lib::persistency::Key;

use embassy_rp::Peri;
use embassy_rp::flash::{self, Flash};
use embassy_rp::peripherals::{FLASH, DMA_CH0};

const FLASH_SIZE: usize = const_str::parse!(env!("FLASH_MEMORY_LENGTH"), usize);

const DEVICE_DATA_START: u32 = const_str::parse!(env!("DEVICE_DATA_RELATIVE_ORIGIN"), u32);
const DEVICE_DATA_LENGTH: u32 = const_str::parse!(env!("DEVICE_DATA_LENGTH"), u32);
const ADDRESS_RANGE: Range<u32> = DEVICE_DATA_START .. (DEVICE_DATA_START + DEVICE_DATA_LENGTH);

type MyFlash = Flash<'static, FLASH, flash::Async, FLASH_SIZE>;
pub type FlashPersistency = Persistency<MyFlash>;
pub type Error = persistency::Error<MyFlash>;

pub fn init<I>(flash: Peri<'static, FLASH>, dma_ch0: Peri<'static, DMA_CH0>, irq: I) -> FlashPersistency
where
    I: embassy_rp::interrupt::typelevel::Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_0, embassy_rp::dma::InterruptHandler<DMA_CH0>> + 'static,
{
    let storage = Flash::new(flash, dma_ch0, irq);
    Persistency::new(storage, ADDRESS_RANGE)
}

#[cfg(test)]
#[embedded_test::tests]
mod tests {
    // Note:
    // Unfortunately embedded-test ignores the panic message when using #[should_panic], so we cannot assert on the message.

    use super::*;
    use lib::persistency::{PersistencyTrait, SerializationError};

    #[cfg(feature = "host-test")]
    use {
        embedded_storage_async::nor_flash::NorFlash,
        embedded_storage_file::{NorMemoryInram, NorMemoryAsync},
    };

    #[cfg(feature = "target-test")]
    use {
        defmt_rtt as _,
        defmt::assert_eq,
        crate::modules::test_setup::DmaIrqs,
    };

    // Mock persistency for host-test.
    // Note: This will overwrite existing definitions. For target-test the existing definitions are used.
    #[cfg(feature = "host-test")]
    type HostTestFlash = NorMemoryAsync<NorMemoryInram<4, 4, 256>>;
    #[cfg(feature = "host-test")]
    type FlashPersistency = Persistency<HostTestFlash>;
    #[cfg(feature = "host-test")]
    type Error = persistency::Error<HostTestFlash>;

    #[init]
    async fn init() -> FlashPersistency {
        #[cfg(feature = "host-test")]
        {
            let in_ram_memory = NorMemoryInram::<4, 4, 256>::new(1024);
            let mut storage = NorMemoryAsync::new(in_ram_memory);
            storage.erase(0, 1024).await.unwrap();
            Persistency::new(storage, 0..1024)
        }

        #[cfg(feature = "target-test")]
        {
            // embassy_rp::bind_interrupts!(struct DmaIrq { DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>; });
            let peripherals = embassy_rp::init(Default::default());
            super::init(peripherals.FLASH, peripherals.DMA_CH0, DmaIrqs)
        }
    }

    #[test]
    fn check_some_constants() {
        // Just out of interest.
        assert_eq!(flash::ERASE_SIZE, 4096);
        assert_eq!(DEVICE_DATA_START, 2088960);
        assert_eq!(DEVICE_DATA_LENGTH, 0x2000);
        assert_eq!(FLASH_SIZE, 2097152);
        assert_eq!(ADDRESS_RANGE, 2088960..2097152);
    }

    // Note: At the moment this test fails because there is already stuff stored there.
    // #[test]
    // #[should_panic]
    // async fn read_uninitialized(mut flash_persistency: FlashPersistency) {
    //     // Note: This test might fail if there is already something at this key in flash.
    //     let mut read_value = [0; 32];
    //     let _read_len = flash_persistency.read(Key::WifiSsid, &mut read_value).await;
    // }

    #[test]
    async fn store_and_read(mut flash_persistency: FlashPersistency) {
        async fn store_read_and_assert(flash_persistency: &mut FlashPersistency, key: Key, value: &[u8]) {
            let mut read_value = [0; 32];
            let _ = flash_persistency.store(key, value).await;
            let read_len = flash_persistency.read(key, &mut read_value).await.unwrap().unwrap();
            assert_eq!(&read_value[..read_len], value);
        }

        let key = Key::WifiPassword;
        store_read_and_assert(&mut flash_persistency, key, "first_pw".as_bytes()).await;
        store_read_and_assert(&mut flash_persistency, key, "a5615asdaee213-++-".as_bytes()).await;
    }

    #[test]
    async fn store_too_long_value(mut flash_persistency: FlashPersistency) {
        let key = Key::MqttBrokerPassword;
        let value = [0; 71]; // This is too long for the buffer size of 70.
        let result = flash_persistency.store(key, &value).await;
        assert_eq!(result.unwrap_err(), Error::SerializationError(SerializationError::BufferTooSmall));
    }
}
