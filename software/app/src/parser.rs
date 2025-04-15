//! This module parses received messages, forwards them accordingly and returns the answer.
//! It EnterBootloader and :doc:`../firmware/persistency`.
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

// #[cfg_attr(test, automock)]
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


// Testing has problems. Commented out for the moment.

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use mockall::predicate::*;

    // #[test]
    // fn test_enter_bootloader() {
    //     //let mut mock_send_message = MockSendMessage::new();
    //     let mut mock_enter_bootloader = MockEnterBootloaderTrait::new();

    //     // mock_send_message.expect_call()
    //     //     .with(eq(b"entering bootloader now\n" as &[u8]))
    //     //     .times(1)
    //     //     .returning(|_| ());

    //     mock_enter_bootloader.expect_call()
    //         .times(1)
    //         .returning(|| ());

    //     let mut parser = Parser::new(
    //         //mock_send_message,
    //         mock_enter_bootloader,
    //         MockPersistencyTrait::new(),
    //     );

    //     let mut answer: [u8; 32] = ['1' as u8; 32];
    //     parser.parse_message(b"enter bootloader", &mut answer);
    //     assert_eq!(&answer[..], b"entering bootloader now1111111111");
    // }

    // #[test]
    // fn test_ping_pong() {
    //     let mut parser = Parser::new(
    //         MockEnterBootloaderTrait::new(),
    //         MockPersistencyTrait::new(),
    //     );

    //     let mut answer: [u8; 32] = ['2' as u8; 32];
    //     parser.parse_message(b"ping", &mut answer);
    //     assert_eq!(&answer[..], b"pong2222222222222222222222222222");
    // }

    // #[test]
    // fn test_store_command() {
    //     let commands = vec![
    //         (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       ValueId::WifiSsid),
    //         (b"wifi_password".as_ref(),        b"12345".as_ref(),         ValueId::WifiPassword),
    //         (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), ValueId::MqttHostIp),
    //         (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     ValueId::MqttBrokerUsername),
    //         (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     ValueId::MqttBrokerPassword),
    //     ];

    //     for (command, value, field) in commands {
    //         let mut mock_persistency = MockPersistencyTrait::new();

    //         mock_persistency.expect_store()
    //             .with(eq(value), eq(field))
    //             .times(1)
    //             .returning(|_, _| Box::pin(async move {}));

    //         let mut parser = Parser::new(
    //             MockEnterBootloaderTrait::new(),
    //             mock_persistency,
    //         );

    //         let mut message = Vec::new();
    //         message.extend_from_slice(b"store ");
    //         message.extend_from_slice(command);
    //         message.extend_from_slice(b" ");
    //         message.extend_from_slice(value);

    //         let mut answer: [u8; 32] = ['\0' as u8; 32];
    //         parser.parse_message(message.as_slice(), &mut answer);
    //         assert_eq!(&answer[..], b"");
    //     }
    // }

    // #[test]
    // fn test_read_command() {
    //     let commands = vec![
    //         (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       DataField::WifiSsid),
    //         (b"wifi_password".as_ref(),        b"12345".as_ref(),         DataField::WifiPassword),
    //         (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), DataField::MqttHostIp),
    //         (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     DataField::MqttBrokerUsername),
    //         (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     DataField::MqttBrokerPassword),
    //     ];

    //     for (command, value, field) in commands {
    //         let mut mock_persistency = MockPersistencyTrait::new();

    //         mock_persistency.expect_read()
    //             .with(eq(field))
    //             .times(1)
    //             .returning(|_| value); //TODO: for any reason returning is not found here. Don't know why.

    //         let mut parser = Parser::new(
    //             MockEnterBootloaderTrait::new(),
    //             mock_persistency,
    //         );

    //         let mut message = Vec::new();
    //         message.extend_from_slice(b"read ");
    //         message.extend_from_slice(command);

    //         let answer = parser.parse_message(message.as_slice());
    //         assert_eq!(answer, b"");
    //     }
    // }

    // #[test]
    // fn test_nothing_to_parse() {
    //     let mut parser = Parser::new(
    //         MockEnterBootloaderTrait::new(),
    //         MockPersistencyTrait::new(),
    //     );

    //     let mut answer: [u8; 32] = ['\0' as u8; 32];
    //     parser.parse_message(b"no command", &mut answer);
    //     assert_eq!(&answer[..], b"nothing to parse\n");
    // }
// }
