#!/usr/bin/env python3

import argparse
import subprocess
import time
import pathlib

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Execute common tasks (building, testing, ...)')

    parser.add_argument('-v', '--verbose',
                        action='store_true',
                        help='verbose output')
    parser.add_argument('-p', '--pseudo_tty_off',
                        action='store_true',
                        help='disable colorfull output')

    commands = parser.add_subparsers(dest='command', required=True)

    doc = commands.add_parser('doc', help='Documentation')
    doc.add_argument('-b', '--build',
                     action='store_true',
                     help='build documentation')
    doc.add_argument('-a', '--autobuild',
                     action='store_true',
                     help='start sphinx-autobuild')

    features = commands.add_parser('features', help='Features')
    features.add_argument('-t', '--test',
                          action='store_true',
                          help='build and run features')

    software = commands.add_parser('software', help='Software')
    software.add_argument('-b', '--build',
                          action='store_true',
                          help='build the software')
    software.add_argument('--target_test', '--tt',
                          action='store_true',
                          help='test the software on the target')
    software.add_argument('--host_test', '--ht',
                          action='store_true',
                          help='test the software on the target')
    software.add_argument('-u', '--upload',
                          action='store_true',
                          help='upload the software to RPI after rebuild')
    software.add_argument('--set_version_from_tag', '--sv',
                          action='store_true',
                          help='set the version in Cargo.toml to the given tag e.g. for release build')

    arguments = parser.parse_args()


    # Note: cd into doc, features, software, ... is not necessary as this step is done by setting the cwd in subprocess.run.
    if arguments.command == 'doc':
        if arguments.build:
            commands = 'make html SPHINXOPTS="--fail-on-warning"'
        elif arguments.autobuild:
            commands = 'sphinx-autobuild '+ ('' if arguments.verbose else '-q') +' --port 8000 --host 0.0.0.0 '
            commands += '--watch ../software/firmware/src --watch ../features '
            commands += '--re-ignore auto_generated source _build/html'

    elif arguments.command == 'features':
        if arguments.test:
            commands = 'cd steps && cargo test'

    elif arguments.command == 'software':
        commands = 'cd targets/rp2040'
        commands += ' && mkdir -p target' # tee needs the folder to exist

        if arguments.build:
            commands += ' && cargo build'
        elif arguments.target_test:
            commands += ' && cargo test --lib --no-default-features --features target-test'
            # commands += ' -- --color always' The color option exists but errors as unexpected argument.
            commands += ' | tee target/target-test-report.txt'
        elif arguments.host_test:
            commands += ' && cargo test --lib --no-default-features --features host-test --target x86_64-unknown-linux-gnu'
            commands += ' -- --color always'
            commands += ' | tee target/host-test-report.txt'
        elif arguments.upload:
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
                            if arguments.verbose:
                                print("Info: Device was sent into bootloader mode.")

            commands = 'cd firmware && cargo run'
        elif arguments.set_version_from_tag:
            commands = 'cd firmware && cargo set-version $(git tag | sed "s/^.//")'

    this_file_dir = pathlib.Path(__file__).resolve().parent
    cwd = this_file_dir / arguments.command
    subprocess.run(commands, cwd=cwd, shell=True, check=True)
