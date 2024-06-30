#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnterBootloader {
    fn call(&mut self);
}

#[cfg_attr(test, automock)]
pub trait Persistency {
    fn store_wifi_ssid(&mut self, wifi_ssid: &[u8]);
    fn store_wifi_password(&mut self, wifi_password: &[u8]);
    fn store_mqtt_host_ip(&mut self, mqtt_host_ip: &[u8]);
    fn store_mqtt_broker_username(&mut self, mqtt_broker_username: &[u8]);
    fn store_mqtt_broker_password(&mut self, mqtt_broker_password: &[u8]);

    fn read_wifi_ssid(&mut self) -> &[u8];
    fn read_wifi_password(&mut self) -> &[u8];
    fn read_mqtt_host_ip(&mut self) -> &[u8];
    fn read_mqtt_broker_username(&mut self) -> &[u8];
    fn read_mqtt_broker_password(&mut self) -> &[u8];
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
            self.persistency.store_wifi_ssid(value);
        }
        else if parameters.starts_with(WIFI_PASSWORD) {
            let value = &parameters[WIFI_PASSWORD.len()..];
            self.persistency.store_wifi_password(value);
        }
        else if parameters.starts_with(MQTT_HOST_IP) {
            let value = &parameters[MQTT_HOST_IP.len()..];
            self.persistency.store_mqtt_host_ip(value);
        }
        else if parameters.starts_with(MQTT_BROKER_USERNAME) {
            let value = &parameters[MQTT_BROKER_USERNAME.len()..];
            self.persistency.store_mqtt_broker_username(value);
        }
        else if parameters.starts_with(MQTT_BROKER_PASSWORD) {
            let value = &parameters[MQTT_BROKER_PASSWORD.len()..];
            self.persistency.store_mqtt_broker_password(value);
        }
        else {
            panic!("unknown parameter");
        }
    }

    fn parse_read_command(&mut self, parameters: &[u8]) -> &[u8]{
        if parameters.starts_with(b"wifi_ssid") {
            return self.persistency.read_wifi_ssid();
        }
        else if parameters.starts_with(b"wifi_password") {
            return self.persistency.read_wifi_password();
        }
        else if parameters.starts_with(b"mqtt_host_ip") {
            return self.persistency.read_mqtt_host_ip();
        }
        else if parameters.starts_with(b"mqtt_broker_username") {
            return self.persistency.read_mqtt_broker_username();
        }
        else if parameters.starts_with(b"mqtt_broker_password") {
            return self.persistency.read_mqtt_broker_password();
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
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref()),
            (b"wifi_password".as_ref(),        b"12345".as_ref()),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref()),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref()),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref()),
        ];

        for (command, value) in commands {
            let mut mock_persistency = MockPersistency::new();

            if command == b"wifi_ssid" {
                mock_persistency.expect_store_wifi_ssid()
                    .with(eq(value))
                    .times(1)
                    .returning(|_| ());
            }
            else if command == b"wifi_password" {
                mock_persistency.expect_store_wifi_password()
                    .with(eq(value))
                    .times(1)
                    .returning(|_| ());
            }
            else if command == b"mqtt_host_ip" {
                mock_persistency.expect_store_mqtt_host_ip()
                    .with(eq(value))
                    .times(1)
                    .returning(|_| ());
            }
            else if command == b"mqtt_broker_username" {
                mock_persistency.expect_store_mqtt_broker_username()
                    .with(eq(value))
                    .times(1)
                    .returning(|_| ());
            }
            else if command == b"mqtt_broker_password" {
                mock_persistency.expect_store_mqtt_broker_password()
                    .with(eq(value))
                    .times(1)
                    .returning(|_| ());
            }
            else {
                panic!("unknown command");
            }

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
    //         (b"wifi_ssid".as_ref(),            b"myValue".as_ref()),
    //         // (b"wifi_password".as_ref(),        b"12345".as_ref()),
    //         // (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref()),
    //         // (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref()),
    //         // (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref()),
    //     ];

    //     for (command, value) in commands {
    //         let mut mock_persistency = MockPersistency::new();

    //         if command == b"wifi_ssid" {
    //             mock_persistency.expect_read_wifi_ssid()
    //                 .times(1)
    //                 .returning(); //TODO: for any reason returning is not found here. No clue what is going on.
    //         }
    //         // else if command == b"wifi_password" {
    //         //     mock_persistency.expect_read_wifi_password()
    //         //         .times(1)
    //         //         .returning(|_| ());
    //         // }
    //         // else if command == b"mqtt_host_ip" {
    //         //     mock_persistency.expect_read_mqtt_host_ip()
    //         //         .times(1)
    //         //         .returning(|_| ());
    //         // }
    //         // else if command == b"mqtt_broker_username" {
    //         //     mock_persistency.expect_read_mqtt_broker_username()
    //         //         .times(1)
    //         //         .returning(|_| ());
    //         // }
    //         // else if command == b"mqtt_broker_password" {
    //         //     mock_persistency.expect_read_mqtt_broker_password()
    //         //         .times(1)
    //         //         .returning(|_| ());
    //         // }
    //         // else {
    //         //     panic!("unknown command");
    //         // }

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
