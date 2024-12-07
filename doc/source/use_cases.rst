Use Cases
=========

.. needflow::
   :show_link_names:
   :config: lefttoright

.. actor:: 5V Power Supply
    :id: A_001
    :association: UC_001

.. actor:: Admin
    :id: A_002
    :association: UC_001, UC_002

.. actor:: 433MHz Sender
    :id: A_003
    :association: UC_003


.. usecase:: Power the Device via USB
    :id: UC_001

.. usecase:: Read and Write Configuration via USB
    :id: UC_002

.. usecase:: Send Button Press to 433MHz Receiver
    :id: UC_003
    :includes: UC_004

.. usecase:: Send Button Press to MQTT Broker
    :id: UC_004
