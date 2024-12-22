import argparse
import subprocess
import sys
import pathlib
import os
import json

sys.path.append(str(pathlib.Path(__file__).parent))
import common

class Executor:
    def __init__(self, additional_arguments, description):
        self._parse_arguments(additional_arguments, description)
        self.work_dir = common.get_caller_path()


    def _parse_arguments(self, additional_arguments, description):
        parser = argparse.ArgumentParser(description)

        for argument in additional_arguments:
            parser.add_argument(argument['flag'], argument['name'],
                                action='store_true',
                                help=argument['help'])

        parser.add_argument('-v', '--verbose',
                            action='store_true',
                            help='Verbose output.')

        parser.add_argument('-k', '--keep_open',
                            action='store_true',
                            help='Enter the command line of the container.')

        parser.add_argument('-p', '--pseudo_tty_off',
                            action='store_true',
                            help='Disable colorfull output.')

        self.arguments = parser.parse_args()


    def run(self, commands):
        if self.arguments.keep_open:
            commands = 'bash'

        workspace_folder = os.path.join(self.work_dir, '..', '..')
        devcontainer_path = os.path.join(self.work_dir, 'devcontainer.json')

        try:
            result = subprocess.run(['devcontainer', 'up', '--workspace-folder', workspace_folder, '--config', devcontainer_path],
                                    check=True, capture_output=True)
            container_info = json.loads(result.stdout)
            containerId = container_info['containerId']

            print('container id: ' + containerId)
            print(result.stdout)

            exec_command = ['devcontainer', 'exec', '--workspace-folder', workspace_folder, '--config', devcontainer_path]
            exec_command.extend(['bash', '-c', 'set -e && ' + commands])
            subprocess.run(exec_command, check=True)

        finally:
            # Stopping the container is a workaround:
            # The RPI Pico volume is not mounted correctly if the container is already running.
            subprocess.run(['docker', 'stop', containerId], check=True)
