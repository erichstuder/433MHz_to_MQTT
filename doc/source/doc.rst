System Scope and Context
========================

433MHz_to_MQTT receives the data from a 433MHz Sender and forwards it to an MQTT Broker.

.. uml::
   @startuml
   component [433MHz Sender]
   component [433MHz_to_MQTT]<<SUD>> #Khaki {
      port 433MHz_Input
      port WiFi
      component [433MHz Receiver]
      component [RPi Pico W]
   }
   component [Network] {
      component [MQTT Broker]
      port MQTT
   }

   [433MHz Sender] --( 433MHz_Antenna
   433MHz_Antenna -- 433MHz_Input
   433MHz_Input -- [433MHz Receiver]
   [433MHz Receiver] -- [RPi Pico W]
   [RPi Pico W] -- WiFi
   WiFi --( WiFi_Antenna
   WiFi_Antenna -- MQTT
   MQTT -- [MQTT Broker]
   @enduml

+-----------------+------------------------------------------------+
| Element         | Description                                    |
+=================+================================================+
| 433MHz_to_MQTT  | Data Gateway from 433MHz to MQTT               |
|                 |                                                |
|                 | System Under Design (SUD)                      |
+-----------------+------------------------------------------------+
| 433MHz Receiver | 433MHz receiver as a separated hardware module |
+-----------------+------------------------------------------------+
| RPi Pico W      | Controller platform containing:                |
|                 |                                                |
|                 | * Software                                     |
|                 | * WiFi hardware                                |
+-----------------+------------------------------------------------+
| 433MHz Sender   | Sender e.g. a remote control                   |
+-----------------+------------------------------------------------+
| MQTT Broker     | Receiver of the data for further processing    |
+-----------------+------------------------------------------------+
