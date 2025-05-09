#!/usr/bin/env python3

import sys
import pathlib
import time

sys.path.append(str(pathlib.Path(__file__).parent.parent / 'project_management'))
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
        commands = 'cd firmware && mkdir -p build && cargo test --no-default-features --features test --target x86_64-unknown-linux-gnu | tee build/unit-test-report.txt'
    elif ex.arguments.upload:
        if ex.running_in_container:
            print("Upload is not (yet) supported inside the container.", file=sys.stderr)
            # The problem is, that the RPI is mounted as owned by root, on which the docker user has no access.
            sys.exit(1)

        # TODO: Maybe we could send the device into bootloader mode directly from inside the container?
        import pyudev # Import only here, as this file is also used on github runners without hardware access. So this is not installed and won't be used there.
        import serial
        udev = pyudev.Context()
        for usb_device in  udev.list_devices(subsystem="usb"):
            if usb_device.attributes.get('manufacturer') == b'github.com/erichstuder' and usb_device.attributes.get('product') == b'433MHz_to_MQTT':
                for tty_device in  udev.list_devices(subsystem="tty"):
                    if tty_device.sys_path.startswith(usb_device.sys_path):
                        my_serial = serial.Serial(None)
                        my_serial.port = tty_device.device_node
                        my_serial.open()
                        my_serial.write("enter bootloader\n".encode())
                        my_serial.close()
                        time.sleep(4) #wait for the device to enter bootloader mode
                        if ex.arguments.verbose:
                            print("Info: Device was sent into bootloader mode.")

        commands = 'cd firmware && cargo run'
    else:
        commands = None

    ex.run(commands)
