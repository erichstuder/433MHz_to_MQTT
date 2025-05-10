import argparse
import subprocess
import sys
import pathlib
import os

sys.path.append(str(pathlib.Path(__file__).parent))
import common

class Executor:
    def __init__(self, additional_arguments, description):
        self._parse_arguments(additional_arguments, description)
        self.work_dir = common.get_caller_path()
        self.running_in_container = os.path.exists('/.dockerenv')


    def _parse_arguments(self, additional_arguments, description):
        parser = argparse.ArgumentParser(description)

        for argument in additional_arguments:
            parser.add_argument(argument['flag'], argument['name'],
                                dest=argument['name'].lstrip('-'),
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

        if self.running_in_container:
            self._run_directly(commands)
        else:
            self._run_with_container(commands)


    def _run_directly(self, commands):
        subprocess.run(commands, shell=True, check=True)


    def _run_with_container(self, commands):
        docker_args = ['bash', '-c', 'set -e \n ' + commands]
        yml_file_path = self.work_dir + '/docker-compose.yml'
        project = 'project_management'
        service_name = 'main'

        env = os.environ.copy()
        env['UID'] = str(os.getuid())

        try:
            subprocess.run(['docker-compose', '-f', yml_file_path, '-p', project, 'up', '--build', '--detach'], check=True, env=env)

            exec_command = ['docker-compose', '-f', yml_file_path, '-p', project, 'exec']
            if self.arguments.pseudo_tty_off:
                exec_command.append('-T')
            exec_command.append(service_name)
            exec_command.extend(docker_args)
            subprocess.run(exec_command, check=True, env=env)

        finally:
            subprocess.run(["docker-compose", '-f', yml_file_path, '-p', project, "down"], check=True, env=env)
