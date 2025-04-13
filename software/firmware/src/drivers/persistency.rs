//! This module handles the persistency.
//! It allows to persistently store and read data.

use embassy_rp::flash::{self, Flash};
use embassy_rp::peripherals::{FLASH, DMA_CH0};
use core::future::Future;
use app::parser;
use crate::PersistencyMutexed;

struct Value {
    id: ValueId,
    length: u8,
    index: usize,
}

impl Value {
    fn new(id: ValueId) -> Self {
        Self {
            id,
            length: 0,
            index: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum ValueId {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

// These values must align with the specifications in memory.x.
const FLASH_SIZE: usize = 2*1024*1024; // 2MB is valid for Raspberry Pi Pico.
const DATA_SIZE: usize = flash::ERASE_SIZE; // must be a multiple of ERASE_SIZE.
const DATA_ADDRESS_OFFSET: usize = FLASH_SIZE - flash::ERASE_SIZE; // put data at the end of flash memory.
const FILE_DESCRIPTOR_SIZE: usize = 5;

pub struct Persistency {
    flash: Flash<'static, FLASH, flash::Async, FLASH_SIZE>,
    values: [Value; FILE_DESCRIPTOR_SIZE],
    data: [u8; DATA_SIZE],
}

impl Persistency {
    pub fn new(flash: FLASH, dma: DMA_CH0) -> Self {
        Self {
            flash: Flash::new(flash, dma),
            values: [
                Value::new(ValueId::WifiSsid),
                Value::new(ValueId::WifiPassword),
                Value::new(ValueId::MqttHostIp),
                Value::new(ValueId::MqttBrokerUsername),
                Value::new(ValueId::MqttBrokerPassword),
            ],
            data: [0; DATA_SIZE],
        }
    }

    fn read_all(&mut self) {
        self.flash.blocking_read(DATA_ADDRESS_OFFSET as u32, &mut self.data).expect("failed to read flash memory");

        for n in 0..self.values.len() {
            self.values[n].length = self.data[n];

            if n == 0 {
                self.values[n].index = FILE_DESCRIPTOR_SIZE;
            } else {
                self.values[n].index = self.values[n-1].index + self.values[n-1].length as usize;
            }
        }
    }

    fn get_value(&self, value_id: ValueId) -> &Value {
        for n in 0..self.values.len() {
            if self.values[n].id == value_id {
                return &self.values[n];
            }
        }
        panic!("value id not found");
    }

    pub fn read(&mut self, value_id: ValueId, answer: &mut [u8; 32]) -> Result<usize, &'static str> {
        self.read_all();

        let value = self.get_value(value_id);
        let length = value.length as usize;
        let index = value.index;

        if length > answer.len(){
            Err("answer buffer too small")
        }
        else {
            answer[..length].copy_from_slice(&self.data[index..(index + length)]);
            Ok(length)
        }
    }

    fn shift_and_set_to_new_length(&mut self, position: usize, new_length: u8) {
        if new_length == self.values[position as usize].length {
            return;
        }

        let index = self.values[position].index;

        if new_length > self.values[position].length {
            let offset = (new_length - self.values[position].length) as usize;
            for n in ((index + offset)..DATA_SIZE).rev() {
                self.data[n] = self.data[n - offset];
            }
        } else {
            let offset = (self.values[position].length - new_length) as usize;
            for n in index..(DATA_SIZE - offset) {
                self.data[n] = self.data[n + offset];
            }
        }
        self.values[position].length = new_length;
        self.data[position] = new_length; //TODO: this looks unclean
    }

    pub fn store(&mut self, value: &[u8], value_id: ValueId) {
        self.read_all();

        let mut position: Option<usize> = None;
        let new_length = value.len() as u8;
        for n in 0..self.values.len() {
            if self.values[n].id == value_id {
                position = Some(n);
                break;
            }
        }
        let position = position.expect("position not found");

        self.shift_and_set_to_new_length(position, new_length);

        let index = self.values[position].index;

        self.data[index..index+value.len()].copy_from_slice(value);

        self.flash.blocking_erase(DATA_ADDRESS_OFFSET as u32, (DATA_ADDRESS_OFFSET + DATA_SIZE) as u32).expect("Failed to erase flash memory.");
        self.flash.blocking_write(DATA_ADDRESS_OFFSET as u32, &self.data).expect("Failed to write flash memory.");
    }
}

pub struct ParserToPersistency {
    persistency_mutexed: &'static PersistencyMutexed,
}
impl ParserToPersistency {
    pub fn new(persistency_mutexed: &'static PersistencyMutexed) -> Self {
        Self {
            persistency_mutexed: persistency_mutexed,
        }
    }
}
impl parser::PersistencyTrait for ParserToPersistency {
    fn store<'a>(&'a mut self, value: &'a [u8], value_id: parser::ValueId) -> impl Future<Output = ()> + 'a {
        async move {
            let mut persistency = self.persistency_mutexed.lock().await;
            match value_id {
                parser::ValueId::WifiSsid           => persistency.store(value, ValueId::WifiSsid),
                parser::ValueId::WifiPassword       => persistency.store(value, ValueId::WifiPassword),
                parser::ValueId::MqttHostIp         => persistency.store(value, ValueId::MqttHostIp),
                parser::ValueId::MqttBrokerUsername => persistency.store(value, ValueId::MqttBrokerUsername),
                parser::ValueId::MqttBrokerPassword => persistency.store(value, ValueId::MqttBrokerPassword),
            }
        }
    }

    fn read<'a>(&'a mut self, value_id: parser::ValueId, answer: &'a mut [u8; 32]) -> impl Future<Output = Result<usize, &'static str>> + 'a {
        async move {
            let mut persistency = self.persistency_mutexed.lock().await;
            match value_id {
                parser::ValueId::WifiSsid           => persistency.read(ValueId::WifiSsid, answer),
                parser::ValueId::WifiPassword       => persistency.read(ValueId::WifiPassword, answer),
                parser::ValueId::MqttHostIp         => persistency.read(ValueId::MqttHostIp, answer),
                parser::ValueId::MqttBrokerUsername => persistency.read(ValueId::MqttBrokerUsername, answer),
                parser::ValueId::MqttBrokerPassword => persistency.read(ValueId::MqttBrokerPassword, answer),
            }
        }
    }
}
