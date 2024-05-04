System Scope and Context
========================

433MHz_to_MQTT receives the pressed Button from a 433MHz Sender (Remote Control) and forwards it to a MQTT Broker.

.. uml::

   @startuml
   skinparam component<<wide>> {
     BackgroundColor Khaki
   }

   component [433MHz_to_MQTT]<<wide>>
   component [433MHz Sender (Remote Control)]
   component [MQTT Broker]

   [433MHz_to_MQTT] <.. [433MHz Sender (Remote Control)] : pressed Button
   [433MHz_to_MQTT] ..> [MQTT Broker] : pressed Button
   @enduml
