# language: en

Feature: RS232
    Scenario Outline: Persisted Parameters
        Given the connection to the device via USB
        Then a serial connection can be established
