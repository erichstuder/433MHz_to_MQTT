# language: en

Feature: RS232
%.. feature:: RS232
%    :id: F_002
%    :links: UC_002

    Scenario Outline: Persisted Parameters
        Given the connection to the device via USB
        Then a serial connection can be established
