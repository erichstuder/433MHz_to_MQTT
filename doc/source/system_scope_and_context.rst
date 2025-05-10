System Scope and Context
========================

433MHz_to_MQTT receives the data from a 433MHz Sender and forwards it to an MQTT Broker.

.. drawio-image:: context_diagram.drawio

.. list-table::
   :header-rows: 1

   * - Element
     - Description

   * - 433MHz_to_MQTT
     - This is the system we develop.

       It Transmitts the data from a 433MHz Sender to

       an MQTT Broker (e.g. a key-press).

   * - 433MHz Receiver
     - - Receiver for the 433MHz signal.
       - It is powered by the RPi Pico W.
       - This component is bought (e.g. `here) <https://de.aliexpress.com/item/1005003436580019.html>`_ as a module.
       - See :doc:`auto_generated/firmware/modules/button_task` for receiver pin connection.
       - .. image:: receiver.png
            :width: 100px

   * - RPi Pico W
     - The Raspberry Pi Pico W receives the data from the 433MHz Receiver

       and forwards it to the MQTT Broker.

   * - 433MHz Sender
     - A remote device that sends a 433MHz signal (e.g. a remote control).

   * - MQTT Broker in the LAN
     - The MQTT Broker is located in a network and receives the data from the

       433MHz_to_MQTT system for further processing

       (e.g. in a home assistant system).

   * - 5V Power Supply
     - The system is powered with 5V via the USB-micro port of the RPi Pico W.

   * - Admin
     - - Aministration is done via the USB-micro port of the RPi Pico W.
       - This interface allows to flash the RPi Pico W and to configure the system.
