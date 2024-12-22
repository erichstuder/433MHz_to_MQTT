#!/usr/bin/env python3

import sys
import pathlib
import serial
import time

sys.path.append(str(pathlib.Path(__file__).parent.parent.parent / 'project_management'))
from executor import Executor # type: ignore


if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-b',
            'name': '--build',
            'help': 'Build the software.'
        },
        {
            'flag': '-t',
            'name': '--test',
            'help': 'Test the software.'
        },
        {
            'flag': '-u',
            'name': '--upload',
            'help': 'Upload the software to RPI after rebuild.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute feature tests')

    if ex.arguments.build:
        commands = 'cd firmware && cargo build'
    elif ex.arguments.test:
        commands = 'cd app && cargo test'
    elif ex.arguments.upload:
        # TODO: Maybe we could send the device into bootloader mode directly from inside the container?
        import pyudev # Import only here, as this file is also used on github runners without hardware access. So this is not installed and won't be used there.
        udev = pyudev.Context()
        for usb_device in  udev.list_devices(subsystem="usb"):
            if usb_device.attributes.get('manufacturer') == b'github.com/erichstuder' and usb_device.attributes.get('product') == b'433MHz_to_MQTT':
                for tty_device in  udev.list_devices(subsystem="tty"):
                    if tty_device.sys_path.startswith(usb_device.sys_path):
                        my_serial = serial.Serial(None)
                        my_serial.port = tty_device.device_node
                        my_serial.open()
                        my_serial.write("enter bootloader".encode())
                        my_serial.close()
                        time.sleep(4) #wait for the device to enter bootloader mode
                        if ex.arguments.verbose:
                            print("Info: Device was sent into bootloader mode.")

        commands = 'cd firmware && cargo run'
    else:
        commands = None

    ex.run(commands)