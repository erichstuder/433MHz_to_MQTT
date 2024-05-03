System Scope and Context
========================

.. uml::

   @startuml
   class 433MHz_to_MQTT {
   }
   class "433MHz Sender (Remote Control)" {
   }
   class "MQTT Broker" {
   }
   "433MHz Sender (Remote Control)" --> 433MHz_to_MQTT
   433MHz_to_MQTT --> "MQTT Broker"
   @enduml
