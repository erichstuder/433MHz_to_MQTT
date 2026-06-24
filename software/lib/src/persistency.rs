//! Handles the persistency.
//! It allows to persistently store data.

use core::ops::Range;
use embedded_storage_async::nor_flash::NorFlash;
use sequential_storage::cache::NoCache;
use sequential_storage::map::{MapStorage, MapConfig};

const DATA_BUFFER_SIZE: usize = 32;

#[repr(u8)]
pub enum Key {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
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

    pub async fn store(&mut self, key: Key, value: &[u8]) {
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        self.map_storage.store_item(&mut data_buffer, &(key as u8), &value).await.unwrap();
    }

    pub async fn read(&mut self, key: Key, value: &mut[u8]) -> usize {
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        let result = self.map_storage.fetch_item::<&[u8]>(&mut data_buffer, &(key as u8)).await.unwrap().unwrap();
        value[..result.len()].copy_from_slice(result);
        result.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use embedded_storage_file::{NorMemoryInram, NorMemoryAsync};

    #[tokio::test]
    async fn dummy() {
        let in_ram_memory = NorMemoryInram::<4, 4, 256>::new(1024);
        let mut storage = NorMemoryAsync::new(in_ram_memory);
        storage.erase(0, 1024).await.unwrap();
        let mut persistency = Persistency::new(storage, 0..1024);

        persistency.store(Key::MqttBrokerPassword, "963".as_bytes()).await;

        let mut buffer = [0; 32];
        let length = persistency.read(Key::MqttBrokerPassword, &mut buffer).await;
        assert_eq!(&buffer[..length], "963".as_bytes());
    }
}
