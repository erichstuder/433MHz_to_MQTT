//! Handles the persistency.
//! It allows to persistently store data.

use core::ops::Range;
use embedded_storage_async::nor_flash::{self, NorFlash};
use sequential_storage::cache::NoCache;

use sequential_storage::map::{MapStorage, MapConfig};
pub use sequential_storage::map::SerializationError;

pub type Error<NF> = sequential_storage::Error<<NF as nor_flash::ErrorType>::Error>;

const DATA_BUFFER_SIZE: usize = 70;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Key {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

#[cfg_attr(test, mockall::automock)]
pub trait PersistencyTrait<NF: NorFlash> {
    #[allow(async_fn_in_trait)]
    async fn store(&mut self, key: Key, value: &[u8]) -> Result<(), Error<NF>>;
    #[allow(async_fn_in_trait)]
    async fn read(&mut self, key: Key, value: &mut[u8]) -> Result<Option<usize>, Error<NF>>;
}

pub struct Persistency<NF: NorFlash> {
    map_storage: MapStorage<u8, NF, NoCache>,
}

impl<NF: NorFlash> Persistency<NF> {
    pub fn new(storage: NF, storage_address_range: Range<u32>) -> Self {
        let map_storage = MapStorage::new(
            storage,
            MapConfig::new(storage_address_range),
            NoCache,
        );
        Self { map_storage }
    }
}

impl<NF: NorFlash> PersistencyTrait<NF> for Persistency<NF> {
    async fn store(&mut self, key: Key, value: &[u8]) -> Result<(), Error<NF>> {
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        self.map_storage.store_item(&mut data_buffer, &(key as u8), &value).await
    }

    async fn read(&mut self, key: Key, value: &mut[u8]) -> Result<Option<usize>, Error<NF>> {
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        if let Some(result) = self.map_storage.fetch_item::<&[u8]>(&mut data_buffer, &(key as u8)).await? {
            if result.len() > value.len() {
                return Err(Error::<NF>::BufferTooSmall(result.len()));
            }
            value[..result.len()].copy_from_slice(result);
            Ok(Some(result.len()))
        } else {
            Ok(None)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use embedded_storage_file::{NorMemoryInram, NorMemoryAsync};

    const READ_SIZE: usize = 4;
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 256;
    type TestFlash = NorMemoryAsync<NorMemoryInram<READ_SIZE, WRITE_SIZE, ERASE_SIZE>>;
    type TestFlashError = Error<TestFlash>;


    async fn setup() -> Persistency<TestFlash> {
        let in_ram_memory = NorMemoryInram::<READ_SIZE, WRITE_SIZE, ERASE_SIZE>::new(1024);
        let mut storage = NorMemoryAsync::new(in_ram_memory);
        storage.erase(0, 1024).await.unwrap();
        Persistency::new(storage, 0..1024)
    }

    #[tokio::test]
    async fn store_and_read() {
        let mut persistency = setup().await;

        let value = "963".as_bytes();

        let result = persistency.store(Key::MqttBrokerPassword, value).await.unwrap();
        assert_eq!(result, ());

        let mut buffer = [0; 32];
        let length = persistency.read(Key::MqttBrokerPassword, &mut buffer).await.unwrap().unwrap();
        assert_eq!(&buffer[..length], value);
    }

    #[tokio::test]
    async fn store_too_long_value() {
        let mut persistency = setup().await;

        let value = [0u8; DATA_BUFFER_SIZE];

        let result = persistency.store(Key::MqttBrokerUsername, &value).await;
        assert_eq!(result.unwrap_err(), TestFlashError::SerializationError(SerializationError::BufferTooSmall));
    }

    #[tokio::test]
    async fn read_non_existent() {
        let mut persistency = setup().await;

        let mut buffer = [0; 32];
        let result = persistency.read(Key::MqttHostIp, &mut buffer).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn read_error() {
        let mut persistency = setup().await;

        const VALUE_SIZE: usize = 20;

        let value = [0u8; VALUE_SIZE];
        let _ = persistency.store(Key::WifiPassword, &value).await;

        let mut buffer = [0; VALUE_SIZE - 1];
        let result = persistency.read(Key::WifiPassword, &mut buffer).await;
        assert_eq!(result.unwrap_err(), TestFlashError::BufferTooSmall(VALUE_SIZE));
    }
}
