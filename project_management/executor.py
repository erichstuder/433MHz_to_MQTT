import argparse
import subprocess
import datetime
import sys
import pathlib

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

        docker_args = ['bash', '-c', 'set -e \n ' + commands]
        yml_file_path = self.work_dir + '/docker-compose.yml'

        subprocess.run(['docker-compose', '-f', yml_file_path, 'up', '--build', '--detach'], check=True)

        exec_command = ['docker-compose', '-f', yml_file_path, 'exec']
        if self.arguments.pseudo_tty_off:
            exec_command.append('-T')
        exec_command.append('main')
        exec_command.extend(docker_args)
        subprocess.run(exec_command, check=True)

        subprocess.run(["docker-compose", '-f', yml_file_path, "down"], check=True)
