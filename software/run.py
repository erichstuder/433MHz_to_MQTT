# You must run this script without sudo. To run docker without sudo do the following:
# sudo groupadd docker
# sudo gpasswd -a $USER docker

import argparse
import os
import subprocess
import datetime
import serial
import time
import pathlib

def parse_arguments():
    parser = argparse.ArgumentParser(description='Execute common software tasks')

    parser.add_argument('--build', '-b',
                        action='store_true',
                        help='Build the software.')

    parser.add_argument('--test', '-t',
                        action='store_true',
                        help='Test the software.')

    parser.add_argument('--upload', '-u',
                        action='store_true',
                        help='Upload the software to RPI after rebuild.')

    parser.add_argument('--pseudo_tty_disable', '-p',
                        action='store_true',
                        help='Disable colorfull output.')

    parser.add_argument('--keep_open', '-k',
                        action='store_true',
                        help='Enter the command line of the container.')

    parser.add_argument('--verbose', '-v',
                        action='store_true',
                        help='Verbose output.')

    global arguments
    arguments = parser.parse_args()


def build_container(container_tag, work_dir):
    args = ['docker', 'build',
        '--tag', container_tag,
        '--build-arg', 'USER=' + os.environ.get('USER'),
        '--build-arg', 'USER_ID=' + str(os.geteuid()),
        '--build-arg', 'GROUP_ID=' + str(os.getegid())]

    if not arguments.verbose:
        args.append('--quiet')
        stdout = subprocess.DEVNULL
    else:
        stdout = None

    args.append(work_dir)

    print('    building container... please wait')
    return subprocess.run(args, stdout=stdout)


def run_container(container_tag, work_dir):

    current_time = datetime.datetime.now().strftime('%Hh_%Mm_%Ss');

    docker_volume_dir = '/usr/433MHz_to_MQTT'

    if arguments.keep_open:
        commands = 'bash'
    elif arguments.upload:
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
                        if arguments.verbose:
                            print("Info: Device was sent into bootloader mode.")

        commands = 'set -e\n cd software/firmware \n cargo run'
    elif arguments.build:
        commands = 'set -e\n cd software/firmware \n cargo build'
    elif arguments.test:
        commands = 'set -e\n cd software/app \n cargo test'
    else:
        return

    return subprocess.run(['docker',
        'run',
        '--rm',
        '--privileged',
        '--name', 'firmware_' + current_time,
        '--volume', '/media/'+os.environ.get('USER')+':/media/user/',
        '--volume', work_dir + '/..:' + docker_volume_dir,
        '--volume', '/dev/bus/usb:/dev/bus/usb',
        '--workdir', docker_volume_dir,
        '-i' + ('' if arguments.pseudo_tty_disable else 't'),
        container_tag,
        'bash', '-c', commands
    ])


def assert_result(result):
    if result is not None and result.returncode != 0:
        if arguments.verbose:
            print(result)
        exit(result.returncode)


def main():
    parse_arguments()

    work_dir = str(pathlib.Path(__file__).parent.resolve())
    container_tag = work_dir[1:].lower().replace('/_', '/')

    if arguments.verbose:
        print('Container Tag: ' + container_tag)

    result = build_container(container_tag, work_dir)
    assert_result(result)

    result = run_container(container_tag, work_dir)
    assert_result(result)


if __name__ == "__main__":
    main()
