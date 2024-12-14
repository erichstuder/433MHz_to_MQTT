#!/usr/bin/env python3

import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent.parent / 'project_management'))
from executor import Executor # type: ignore


if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-t',
            'name': '--test',
            'help': 'Build and run features.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute feature tests')

    if ex.arguments.test:
        commands = 'cd steps && cargo test'
    else:
        commands = None

    ex.run(commands)



# import argparse
# import os
# import subprocess
# import datetime
# import pathlib

# def parse_arguments():
#     parser = argparse.ArgumentParser(description='Execute feature files')

#     parser.add_argument('--run', '-r',
#                         action='store_true',
#                         help='Run tests.')

#     parser.add_argument('--pseudo_tty_disable', '-p',
#                         action='store_true',
#                         help='Disable colorfull output.')

#     parser.add_argument('--keep_open', '-k',
#                         action='store_true',
#                         help='Enter the command line of the container.')

#     parser.add_argument('--verbose', '-v',
#                         action='store_true',
#                         help='Verbose output.')

#     global arguments
#     arguments = parser.parse_args()


# def build_container(container_tag, work_dir):
#     args = ['docker', 'build',
#         '--tag', container_tag]

#     if not arguments.verbose:
#         args.append('--quiet')
#         stdout = subprocess.DEVNULL
#     else:
#         stdout = None

#     args.append(work_dir)

#     print('    building container... please wait')
#     return subprocess.run(args, stdout=stdout)


# def run_container(container_tag, work_dir):
#     current_time = datetime.datetime.now().strftime('%Hh_%Mm_%Ss');
#     docker_volume_dir = '/usr/433MHz_to_MQTT'

#     if arguments.keep_open:
#         commands = 'bash'
#     elif arguments.run:
#         commands = 'set -e\n cd features/steps \n cargo test'
#     else:
#         return

#     return subprocess.run(['docker',
#         'run',
#         '--rm',
#         '--privileged',
#         '--name', 'firmware_' + current_time,
#         '--volume', '/media/'+os.environ.get('USER')+':/media/user/',
#         '--volume', work_dir + '/..:' + docker_volume_dir,
#         '--volume', '/dev/bus/usb:/dev/bus/usb',
#         '--workdir', docker_volume_dir,
#         '-i' + ('' if arguments.pseudo_tty_disable else 't'),
#         container_tag,
#         'bash', '-c', commands
#     ])


# def assert_result(result):
#     if result is not None and result.returncode != 0:
#         if arguments.verbose:
#             print(result)
#         exit(result.returncode)


# def main():
#     parse_arguments()

#     work_dir = str(pathlib.Path(__file__).parent.resolve())
#     container_tag = work_dir[1:].lower().replace('/_', '/')

#     if arguments.verbose:
#         print('Container Tag: ' + container_tag)

#     result = build_container(container_tag, work_dir)
#     assert_result(result)

#     result = run_container(container_tag, work_dir)
#     assert_result(result)


# if __name__ == "__main__":
#     main()
