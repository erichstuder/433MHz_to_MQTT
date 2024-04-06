# You must run this script without sudo. To run docker without sudo do the following:
# sudo groupadd docker
# sudo gpasswd -a $USER docker

import argparse
import os
import subprocess
import datetime
import serial
import time

def parse_arguments():
	parser = argparse.ArgumentParser(description='run cucumber')

	parser.add_argument('--build', '-b',
	                    action='store_true',
	                    help='Build the project.')

	parser.add_argument('--upload', '-u',
	                    action='store_true',
	                    help='Upload the project to RPI after rebuild.')

	parser.add_argument('--pseudo_tty', '-p',
	                    action='store_true',
	                    help='Colorfull output.')

	parser.add_argument('--keep_open', '-k',
	                    action='store_true',
	                    help='Enter the command line of the container.')

	parser.add_argument('--verbose', '-v',
	                    action='store_true',
	                    help='Enable verbose output.')

	global arguments
	arguments = parser.parse_args()


def build_container(container_tag):
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

	args.extend(['--file', 'Dockerfile', '.'])

	print('    building... please wait')
	return subprocess.run(args, stdout=stdout)


def run_container(container_tag):

	current_time = datetime.datetime.now().strftime('%Hh_%Mm_%Ss');

	docker_volume_dir = '/usr/firmware'
	host_volume_dir = os.getcwd()

	if arguments.keep_open:
		commands = 'bash'
	elif arguments.upload:
		# TODO: Maybe we could send the device into bootloader mode directly from inside the container?
		import pyudev # Import only here, as this file is only used on github runners without hw-access.
		udev = pyudev.Context()
		for usb_device in  udev.list_devices(subsystem="usb"):
			if usb_device.attributes.get('manufacturer') == b'github.com/erichstuder' and usb_device.attributes.get('product') == b'433MHz_to_MQTT':
				for tty_device in  udev.list_devices(subsystem="tty"):
					if tty_device.sys_path.startswith(usb_device.sys_path):
						my_serial = serial.Serial(None)
						my_serial.port = tty_device.device_node
						my_serial.open()
						my_serial.write("bootloader".encode())
						my_serial.close()
						time.sleep(4) #wait for the device to enter bootloader mode
						print("Info: Device was sent into bootloader mode.")

		commands = 'set -e\n cargo run'
	elif arguments.build:
		commands = 'set -e\n cargo build'
	else:
		return #do nothing

	return subprocess.run(['docker',
		'run',
		'--rm',
		'--privileged',
		'--name', 'firmware_' + current_time,
		'--volume', '/media/'+os.environ.get('USER')+':/media/user/',
		'--volume', host_volume_dir + ':' + docker_volume_dir,
		'--volume', '/dev/bus/usb:/dev/bus/usb',
		'--workdir', docker_volume_dir,
		'-i' + ('t' if arguments.pseudo_tty else ''),
		container_tag,
		'bash', '-c', commands
	])


def assert_result(result):
	if result.returncode != 0:
		if arguments.verbose:
			print(result)
		exit(result.returncode)


def main():
	parse_arguments()

	container_tag = os.getcwd()[1:].lower().replace('/_', '/')

	if arguments.verbose:
		print('Container Tag: ' + container_tag)

	result = build_container(container_tag)
	assert_result(result)

	result = run_container(container_tag)
	assert_result(result)


if __name__ == "__main__":
    main()
