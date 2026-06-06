#!/usr/bin/env python3

import argparse
import subprocess
import sys
import pathlib
import os

if __name__ == '__main__':
    running_in_container = os.path.exists('/.dockerenv')

    parser = argparse.ArgumentParser(
        description='Execute common tasks (building, testing, ...)'
    )

    parser.add_argument('-v', '--verbose',
                        action='store_true',
                        help='verbose output')
    parser.add_argument('-k', '--keep_open',
                        action='store_true',
                        help='enter the command line of the container')
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
    software.add_argument('-t', '--test',
                          action='store_true',
                          help='test the software')
    software.add_argument('-u', '--upload',
                          action='store_true',
                          help='upload the software to RPI after rebuild')
    software.add_argument('--sv','--set_version_from_tag',
                          action='store_true',
                          help='set the version in Cargo.toml to the given tag e.g. for release build')

    arguments = parser.parse_args()

    if not running_in_container: # then forward the command into the container
        yml_file_path = str(pathlib.Path(__file__).resolve().parent / '.devcontainer' / 'docker-compose.yml')
        project = 'project_management'
        service_name = 'main'
        try:
            subprocess.run(['docker-compose', '-f', yml_file_path, '-p', project, 'up', '--build', '--detach'], check=True)
            exec_command = ['docker-compose', '-f', yml_file_path, '-p', project, 'exec']
            if arguments.pseudo_tty_off:
                exec_command.append('-T')
            exec_command.extend([
                service_name,
                *sys.argv,
            ])
            subprocess.run(exec_command, check=True)
        finally:
            subprocess.run(['docker-compose', '-f', yml_file_path, '-p', project, 'down'], check=True)
    else:
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
            if arguments.build:
                commands = 'cd firmware && cargo build'
            elif arguments.test:
                commands = 'cd firmware && mkdir -p build && cargo test --no-default-features --features test --target x86_64-unknown-linux-gnu | tee build/unit-test-report.txt'
            elif arguments.upload:
                if running_in_container:
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
            elif arguments.set_version_from_tag:
                commands = 'cd firmware && cargo set-version $(git tag | sed "s/^.//")'

        subprocess.run(commands, cwd=arguments.command, shell=True, check=True)
