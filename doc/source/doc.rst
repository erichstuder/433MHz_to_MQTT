System Scope and Context
========================

433MHz_to_MQTT receives the data from a 433MHz Sender and forwards it to a MQTT Broker.

.. uml::

   @startuml
   skinparam component<<SUD>> {
     BackgroundColor Khaki
   }

   component [433MHz Sender]
   component [MQTT Broker]
   component [433MHz_to_MQTT]<<SUD>> {
      component [433MHz Receiver]
      component [RPi Pico W]
   }

   [433MHz Sender] .r.> [433MHz Receiver] : data
   [433MHz Receiver] .r.> [RPi Pico W] : data
   [RPi Pico W] .r.> [MQTT Broker] : data
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
