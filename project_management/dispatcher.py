import argparse
import subprocess
import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent))
import common

class Dispatcher:
    def __init__(self, scripts, description=None):
        if description is None:
            description = 'Execute common tasks (building, testing, ...)'
        self.scripts = scripts
        self.parser = argparse.ArgumentParser(description=description)
        self.group = self.parser.add_mutually_exclusive_group(required=True)
        for key, value in self.scripts.items():
            self.group.add_argument('--'+key,
                                    nargs=argparse.REMAINDER,
                                    help='Pass the remaining arguments to ' + value + '.')
        self.group.add_argument('--ha', '--help_all', dest='help_all',
                                action='store_true',
                                help='Show help for this and all direct subscripts.')


    def parse_args(self):
        self.arguments = self.parser.parse_args()
        work_dir = common.get_caller_path()

        if self.arguments.help_all:
            run_command([work_dir + '/run.py', '--help'])
            print('\n\n*** below are the help messages of the subscripts ***')
            for _, value in self.scripts.items():
                print('\n\n*** ' + value + ' ***')
                sys.stdout.flush()
                run_command([work_dir + '/' + value, '--help'])

        else:
            for key in self.scripts:
                script_args = getattr(self.arguments, key)
                if script_args is not None:
                    run_command([work_dir + '/' + self.scripts[key]] + script_args)
                    break

        return work_dir


def run_dispatcher(scripts, description=None):
    dispatcher = Dispatcher(scripts, description)
    dispatcher.parse_args()
    exit(0)


def run_command(command):
    subprocess.run(command, check=True)
