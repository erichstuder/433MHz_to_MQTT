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
	parser = argparse.ArgumentParser(description='make doc')

	parser.add_argument('--sphinx_autobuild', '--sa',
	                    action='store_true',
	                    help='Starts sphinx-autobuild.')

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


def build_container(container_tag):
	args = ['docker', 'build',
		'--tag', container_tag]

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

	docker_volume_dir = '/usr/doc'
	host_volume_dir = os.getcwd()

	if arguments.keep_open:
		commands = 'bash'
	elif arguments.sphinx_autobuild:
		commands = 'set -e \n sphinx-autobuild --port 8000 --host 0.0.0.0 source _build/html'
	else:
		commands = 'set -e \n make html'

	return subprocess.run(['docker',
		'run',
		'--rm',
		'--name', 'doc_' + current_time,
		'--publish', '8000:8000', # for sphinx-autobuild
		'--publish', '35729:35729', # for sphinx-autobuild
		'--volume', host_volume_dir + ':' + docker_volume_dir,
		'--workdir', docker_volume_dir,
		'-i' + ('' if arguments.pseudo_tty_disable else 't'),
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
