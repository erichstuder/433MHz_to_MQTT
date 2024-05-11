# You must run this script without sudo. To run docker without sudo do the following:
# sudo groupadd docker
# sudo gpasswd -a $USER docker

import argparse
import os
import subprocess
import datetime
import pathlib

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


def build_container(container_tag, work_dir):
	args = ['docker', 'build',
		'--tag', container_tag]

	if not arguments.verbose:
		args.append('--quiet')
		stdout = subprocess.DEVNULL
	else:
		stdout = None

	args.extend(['--file', work_dir+'/Dockerfile', '.'])

	print('    building... please wait')
	return subprocess.run(args, stdout=stdout)


def run_container(container_tag, work_dir):

	current_time = datetime.datetime.now().strftime('%Hh_%Mm_%Ss');

	docker_volume_dir = '/usr/project'

	work_dir_commands = 'set -e \n cd doc \n'

	if arguments.keep_open:
		commands = 'bash'
	elif arguments.sphinx_autobuild:
		commands = work_dir_commands + 'sphinx-autobuild '+ ('' if arguments.verbose else '-q') +' --port 8000 --host 0.0.0.0 source _build/html'
	else:
		commands = work_dir_commands + 'make html'

	return subprocess.run(['docker',
		'run',
		'--rm',
		'--name', 'doc_' + current_time,
		'--publish', '8000:8000', # for sphinx-autobuild
		'--publish', '35729:35729', # for sphinx-autobuild
		'--volume', work_dir + '/..:' + docker_volume_dir,
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
