# language: en

Feature: Persistency
%.. feature:: Persistency
%    :id: F_001
%    :links: UC_002
%
    Note: All messages and answers end with a newline character. See '\\n' below.

    Note: Some examples are commented out and may not be visible in the documentation. For the moment only one example at once can be run. see: https://github.com/cucumber-rs/cucumber/issues/340


    Scenario Outline: Persisted Parameters
        Given the communication to the device over RS232
        When the command is sent: 'store <parameter_name> <value_example>\\n'
        # And the device is power cycled
        And the command is sent: 'read <parameter_name>\\n'
        Then the answer is: '<value_example>\\n'

        Examples:
        | parameter            | parameter_name       | value_example   |
        # | Wi-Fi SSID           | wifi_ssid            | this_is_an_ssid |
        # | Wi-Fi Password       | wifi_password        | wifi_password   |
        # | MQTT Host IP         | mqtt_host_ip         | 123.456.78.9    |
        # | MQTT Broker Username | mqtt_broker_username | username_123    |
        | MQTT Broker Password | mqtt_broker_password | mqtt_password   |
