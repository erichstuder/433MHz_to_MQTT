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

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnterBootloader {
    fn call(&mut self);
}

#[derive(Debug, PartialEq)]
pub enum ValueId {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

#[cfg_attr(test, automock)]
pub trait Persistency {
    fn store(&mut self, value: &[u8], field: ValueId);
    fn read(&mut self, field: ValueId) -> &[u8];
}

pub struct Parser<E: EnterBootloader, P: Persistency> {
    enter_bootloader: E,
    persistency: P,
}

impl<E: EnterBootloader, P: Persistency> Parser<E, P> {
    pub fn new(enter_bootloader: E, persistency: P) -> Self {
        Self {
            enter_bootloader,
            persistency,
        }
    }

    fn parse_store_command(&mut self, parameters: &[u8]) {
        const WIFI_SSID: &[u8] = b"wifi_ssid ";
        const WIFI_PASSWORD: &[u8] = b"wifi_password ";
        const MQTT_HOST_IP: &[u8] = b"mqtt_host_ip ";
        const MQTT_BROKER_USERNAME : &[u8] = b"mqtt_broker_username ";
        const MQTT_BROKER_PASSWORD : &[u8] = b"mqtt_broker_password ";

        if parameters.starts_with(WIFI_SSID) {
            let value = &parameters[WIFI_SSID.len()..];
            self.persistency.store(value, ValueId::WifiSsid);
        }
        else if parameters.starts_with(WIFI_PASSWORD) {
            let value = &parameters[WIFI_PASSWORD.len()..];
            self.persistency.store(value, ValueId::WifiPassword);
        }
        else if parameters.starts_with(MQTT_HOST_IP) {
            let value = &parameters[MQTT_HOST_IP.len()..];
            self.persistency.store(value, ValueId::MqttHostIp);
        }
        else if parameters.starts_with(MQTT_BROKER_USERNAME) {
            let value = &parameters[MQTT_BROKER_USERNAME.len()..];
            self.persistency.store(value, ValueId::MqttBrokerUsername);
        }
        else if parameters.starts_with(MQTT_BROKER_PASSWORD) {
            let value = &parameters[MQTT_BROKER_PASSWORD.len()..];
            self.persistency.store(value, ValueId::MqttBrokerPassword);
        }
        else {
            panic!("unknown parameter");
        }
    }

    fn parse_read_command(&mut self, parameters: &[u8]) -> &[u8]{
        if parameters.starts_with(b"wifi_ssid") {
            return self.persistency.read(ValueId::WifiSsid);
        }
        else if parameters.starts_with(b"wifi_password") {
            return self.persistency.read(ValueId::WifiPassword);
        }
        else if parameters.starts_with(b"mqtt_host_ip") {
            return self.persistency.read(ValueId::MqttHostIp);
        }
        else if parameters.starts_with(b"mqtt_broker_username") {
            return self.persistency.read(ValueId::MqttBrokerUsername);
        }
        else if parameters.starts_with(b"mqtt_broker_password") {
            return self.persistency.read(ValueId::MqttBrokerPassword);
        }
        else {
            panic!("unknown parameter");
        }
    }

    pub fn parse_message(&mut self, msg: &[u8]) -> &[u8] {
        const STORE_COMMAND: &[u8] = b"store ";
        const READ_COMMAND: &[u8] = b"read ";

        if msg == b"enter bootloader" {
            self.enter_bootloader.call();
            // Note: probably this message won't be seen, because of immediate restart.
            b"entering bootloader now\n"
        } else if msg == b"ping" {
            b"pong\n"
        } else if msg.starts_with(STORE_COMMAND) {
            let parameters = &msg[STORE_COMMAND.len()..];
            self.parse_store_command(parameters);
            b""
        } else if msg.starts_with(READ_COMMAND) {
            let parameters = &msg[READ_COMMAND.len()..];
            self.parse_read_command(parameters)
        } else {
            b"nothing to parse\n"
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_enter_bootloader() {
        //let mut mock_send_message = MockSendMessage::new();
        let mut mock_enter_bootloader = MockEnterBootloader::new();

        // mock_send_message.expect_call()
        //     .with(eq(b"entering bootloader now\n" as &[u8]))
        //     .times(1)
        //     .returning(|_| ());

        mock_enter_bootloader.expect_call()
            .times(1)
            .returning(|| ());

        let mut parser = Parser::new(
            //mock_send_message,
            mock_enter_bootloader,
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"enter bootloader");
        assert_eq!(answer, b"entering bootloader now\n");
    }

    #[test]
    fn test_ping_pong() {
        let mut parser = Parser::new(
            MockEnterBootloader::new(),
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"ping");
        assert_eq!(answer, b"pong\n");
    }

    #[test]
    fn test_store_command() {
        let commands = vec![
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       DataField::WifiSsid),
            (b"wifi_password".as_ref(),        b"12345".as_ref(),         DataField::WifiPassword),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), DataField::MqttHostIp),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     DataField::MqttBrokerUsername),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     DataField::MqttBrokerPassword),
        ];

        for (command, value, field) in commands {
            let mut mock_persistency = MockPersistency::new();

            mock_persistency.expect_store()
                .with(eq(value), eq(field))
                .times(1)
                .returning(|_, _| ());

            let mut parser = Parser::new(
                MockEnterBootloader::new(),
                mock_persistency,
            );

            let mut message = Vec::new();
            message.extend_from_slice(b"store ");
            message.extend_from_slice(command);
            message.extend_from_slice(b" ");
            message.extend_from_slice(value);

            let answer = parser.parse_message(message.as_slice());
            assert_eq!(answer, b"");
        }
    }

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
    //         let mut mock_persistency = MockPersistency::new();

    //         mock_persistency.expect_read()
    //             .with(eq(field))
    //             .times(1)
    //             .returning(|_| value); //TODO: for any reason returning is not found here. Don't know why.

    //         let mut parser = Parser::new(
    //             MockEnterBootloader::new(),
    //             mock_persistency,
    //         );

    //         let mut message = Vec::new();
    //         message.extend_from_slice(b"read ");
    //         message.extend_from_slice(command);

    //         let answer = parser.parse_message(message.as_slice());
    //         assert_eq!(answer, b"");
    //     }
    // }

    #[test]
    fn test_nothing_to_parse() {
        let mut parser = Parser::new(
            MockEnterBootloader::new(),
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"no command");
        assert_eq!(answer, b"nothing to parse\n");
    }
}
