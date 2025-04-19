//! This module parses received messages, forwards them accordingly and returns the answer.
//! It EnterBootloader and :doc:`persistency`.
//!
//! .. plantuml::
//!
//!    @startuml
//!
//!    Parser o-- EnterBootloader
//!    Parser o-- Persistency
//!
//!    @enduml

use core::future::Future;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnterBootloaderTrait {
    fn call(&mut self);
}

pub trait PersistencyTrait{
    fn store<'a>(&'a mut self, value: &'a [u8], field: ValueId) -> impl Future<Output = ()> + 'a;
    fn read<'a>(&'a mut self, field: ValueId, answer: &'a mut [u8]) -> impl Future<Output = Result<usize, &'static str>> + 'a;
}

#[derive(Debug, PartialEq)]
pub enum ValueId {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

pub struct Parser<E: EnterBootloaderTrait, P: PersistencyTrait> {
    enter_bootloader: E,
    persistency: P,
}

impl<E: EnterBootloaderTrait, P: PersistencyTrait> Parser<E, P> {
    pub fn new(enter_bootloader: E, persistency: P) -> Self {
        Self {
            enter_bootloader,
            persistency,
        }
    }

    async fn parse_store_command(&mut self, parameters: &[u8]) -> Result<(), &'static str> {
        const WIFI_SSID: &[u8] = b"wifi_ssid ";
        const WIFI_PASSWORD: &[u8] = b"wifi_password ";
        const MQTT_HOST_IP: &[u8] = b"mqtt_host_ip ";
        const MQTT_BROKER_USERNAME : &[u8] = b"mqtt_broker_username ";
        const MQTT_BROKER_PASSWORD : &[u8] = b"mqtt_broker_password ";

        if parameters.starts_with(WIFI_SSID) {
            let value = &parameters[WIFI_SSID.len()..];
            self.persistency.store(value, ValueId::WifiSsid).await;
            Ok(())
        }
        else if parameters.starts_with(WIFI_PASSWORD) {
            let value = &parameters[WIFI_PASSWORD.len()..];
            self.persistency.store(value, ValueId::WifiPassword).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_HOST_IP) {
            let value = &parameters[MQTT_HOST_IP.len()..];
            self.persistency.store(value, ValueId::MqttHostIp).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_USERNAME) {
            let value = &parameters[MQTT_BROKER_USERNAME.len()..];
            self.persistency.store(value, ValueId::MqttBrokerUsername).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_PASSWORD) {
            let value = &parameters[MQTT_BROKER_PASSWORD.len()..];
            self.persistency.store(value, ValueId::MqttBrokerPassword).await;
            Ok(())
        }
        else {
            Err("unknown store parameter")
        }
    }

    async fn parse_read_command(&mut self, parameters: &[u8], answer: &mut [u8]) -> Result<usize, &'static str> {
        if parameters.starts_with(b"wifi_ssid") {
            self.persistency.read(ValueId::WifiSsid, answer).await
        }
        else if parameters.starts_with(b"wifi_password") {
            self.persistency.read(ValueId::WifiPassword, answer).await
        }
        else if parameters.starts_with(b"mqtt_host_ip") {
            self.persistency.read(ValueId::MqttHostIp, answer).await
        }
        else if parameters.starts_with(b"mqtt_broker_username") {
            self.persistency.read(ValueId::MqttBrokerUsername, answer).await
        }
        else if parameters.starts_with(b"mqtt_broker_password") {
            self.persistency.read(ValueId::MqttBrokerPassword, answer).await
        }
        else {
            Err("unknown read parameter")
        }
    }

    fn copy_to_beginning(dest: &mut [u8], src: &[u8]) -> usize {
        let len = src.len().min(dest.len());
        dest[..len].copy_from_slice(&src[..len]);
        len
    }

    pub async fn parse_message(&mut self, msg: &[u8], answer: &mut [u8]) -> Result<usize, &'static str> {
        const STORE_COMMAND: &[u8] = b"store ";
        const READ_COMMAND: &[u8] = b"read ";
        if msg == b"enter bootloader" {
            self.enter_bootloader.call();
            // Note: probably this message won't be seen, because of immediate restart.
            Ok(Self::copy_to_beginning(answer, b"entering bootloader now"))
        } else if msg == b"ping" {
            Ok(Self::copy_to_beginning(answer, b"pong"))
        } else if msg.starts_with(STORE_COMMAND) {
            let parameters = &msg[STORE_COMMAND.len()..];
            match self.parse_store_command(parameters).await {
                Ok(_) => Ok(0),
                Err(e) => Err(e),
            }
        } else if msg.starts_with(READ_COMMAND) {
            let parameters = &msg[READ_COMMAND.len()..];
            self.parse_read_command(parameters, answer).await
        } else {
            Ok(Self::copy_to_beginning(answer, b"nothing to parse"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use mockall::predicate::*;
    use std::sync::{Arc, Mutex};

    struct MockPersistency {
        store_call_count: Arc<Mutex<u8>>,
        last_store_value_id: Arc<Mutex<Option<ValueId>>>,
        last_store_value: Arc<Mutex<Option<Vec<u8>>>>,

        read_call_count: Arc<Mutex<u8>>,
        last_read_value_id: Arc<Mutex<Option<ValueId>>>,
        read_value_return: Arc<Mutex<Option<Vec<u8>>>>,
    }

    impl MockPersistency {
        fn new() -> Self {
            Self {
                store_call_count: Arc::new(Mutex::new(0)),
                last_store_value_id: Arc::new(Mutex::new(None)),
                last_store_value: Arc::new(Mutex::new(None)),

                read_call_count: Arc::new(Mutex::new(0)),
                last_read_value_id: Arc::new(Mutex::new(None)),
                read_value_return: Arc::new(Mutex::new(None)),
            }
        }

        fn get_store_infos(&self) -> (Arc<Mutex<u8>>, Arc<Mutex<Option<ValueId>>>, Arc<Mutex<Option<Vec<u8>>>>) {
            (Arc::clone(&self.store_call_count), Arc::clone(&self.last_store_value_id), Arc::clone(&self.last_store_value))
        }

        fn get_read_infos(&self) -> (Arc<Mutex<u8>>, Arc<Mutex<Option<ValueId>>>) {
            (Arc::clone(&self.read_call_count), Arc::clone(&self.last_read_value_id))
        }

        fn set_read_value_return(&self, value: Vec<u8>) {
            let mut read_value = self.read_value_return.lock().unwrap();
            *read_value = Some(value);
        }
    }

    impl PersistencyTrait for MockPersistency {
        fn store<'a>(&'a mut self, value: &'a [u8], value_id: ValueId) -> impl Future<Output = ()> + 'a {
            let store_call_count = Arc::clone(&self.store_call_count);
            let mut count = store_call_count.lock().unwrap();
            *count += 1;

            let last_store_value_id = Arc::clone(&self.last_store_value_id);
            let mut last_value = last_store_value_id.lock().unwrap();
            *last_value = Some(value_id);

            let last_store_value = Arc::clone(&self.last_store_value);
            let mut last_value = last_store_value.lock().unwrap();
            *last_value = Some(Vec::from(value));

            async move {}
        }

        fn read<'a>(&'a mut self, value_id: ValueId, answer: &'a mut [u8]) -> impl Future<Output = Result<usize, &'static str>> + 'a {
            let read_call_count = Arc::clone(&self.read_call_count);
            let mut count = read_call_count.lock().unwrap();
            *count += 1;

            let last_read_value_id = Arc::clone(&self.last_read_value_id);
            let mut last_value = last_read_value_id.lock().unwrap();
            *last_value = Some(value_id);

            async move {
                let read_value_return = Arc::clone(&self.read_value_return);
                let read_value = read_value_return.lock().unwrap();

                if let Some(ref value) = *read_value {
                    let len = value.len().min(answer.len()); // Ensure we don't overflow the `answer` buffer
                    answer[..len].copy_from_slice(&value[..len]);
                    Ok(len) // Return the number of bytes copied
                } else {
                    Err("No value set for read")
                }
            }
        }
    }

    #[tokio::test]
    async fn test_enter_bootloader() {
        //let mut mock_send_message = MockSendMessage::new();
        let mut mock_enter_bootloader = MockEnterBootloaderTrait::new();

        mock_enter_bootloader.expect_call()
            .times(1)
            .returning(|| ());

        let mut parser = Parser::new(
            mock_enter_bootloader,
            MockPersistency::new(),
        );

        let mut answer: [u8; 32] = ['1' as u8; 32];
        let length = parser.parse_message(b"enter bootloader", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"entering bootloader now");
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let mut parser = Parser::new(
            MockEnterBootloaderTrait::new(),
            MockPersistency::new(),
        );

        let mut answer: [u8; 32] = ['2' as u8; 32];
        let length = parser.parse_message(b"ping", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"pong");
    }

    #[tokio::test]
    async fn test_store_command() {
        let commands = vec![
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       ValueId::WifiSsid),
            (b"wifi_password".as_ref(),        b"12345".as_ref(),         ValueId::WifiPassword),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), ValueId::MqttHostIp),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     ValueId::MqttBrokerUsername),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     ValueId::MqttBrokerPassword),
        ];

        for (command, value, value_id) in commands {
            let mock_persistency = MockPersistency::new();
            let (store_call_count, last_store_value_id, last_store_value) = mock_persistency.get_store_infos();

            let mut parser = Parser::new(
                MockEnterBootloaderTrait::new(),
                mock_persistency,
            );

            let mut message = Vec::new();
            message.extend_from_slice(b"store ");
            message.extend_from_slice(command);
            message.extend_from_slice(b" ");
            message.extend_from_slice(value);

            let mut answer = ['\0' as u8; 0];
            let length = parser.parse_message(message.as_slice(), &mut answer).await.unwrap();
            assert_eq!(&answer[..length], b"");
            assert_eq!(*store_call_count.lock().unwrap(), 1);
            assert_eq!(*last_store_value_id.lock().unwrap(), Some(value_id));
            assert_eq!(*last_store_value.lock().unwrap(), Some(Vec::from(value)));
        }
    }

    #[tokio::test]
    async fn test_read_command() {
        let commands = vec![
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       ValueId::WifiSsid),
            (b"wifi_password".as_ref(),        b"12345".as_ref(),         ValueId::WifiPassword),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), ValueId::MqttHostIp),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     ValueId::MqttBrokerUsername),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     ValueId::MqttBrokerPassword),
        ];

        for (command, value, value_id) in commands {
            let mock_persistency = MockPersistency::new();
            let (read_call_count, last_read_value_id) = mock_persistency.get_read_infos();
            mock_persistency.set_read_value_return(Vec::from(value));

            let mut parser = Parser::new(
                MockEnterBootloaderTrait::new(),
                mock_persistency,
            );

            let mut message = Vec::new();
            message.extend_from_slice(b"read ");
            message.extend_from_slice(command);

            let mut answer = ['\0' as u8; 100];
            let length = parser.parse_message(message.as_slice(), &mut answer).await.unwrap();
            assert_eq!(&answer[..length], value);
            assert_eq!(*read_call_count.lock().unwrap(), 1);
            assert_eq!(*last_read_value_id.lock().unwrap(), Some(value_id));
        }
    }

    #[tokio::test]
    async fn test_nothing_to_parse() {
        let mut parser = Parser::new(
            MockEnterBootloaderTrait::new(),
            MockPersistency::new(),
        );

        let mut answer = ['\0' as u8; 20];
        let length = parser.parse_message(b"no command", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"nothing to parse");
    }
}
