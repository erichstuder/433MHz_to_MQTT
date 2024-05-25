import argparse
import subprocess
import pathlib
import sys

software_run_py = 'software/run.py'
doc_run_py = 'doc/run.py'


def main():
    parser = argparse.ArgumentParser(description='Execute common tasks like building, testing, uploading, ...')
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--software', '-s',
                        nargs=argparse.REMAINDER,
                        help='Pass the remaining arguments to ' + software_run_py + '.')
    group.add_argument('--doc', '-d',
                        nargs=argparse.REMAINDER,
                        help='Pass the remaining arguments to ' + doc_run_py + '.')
    group.add_argument('--help_all', '--ha',
                        action='store_true',
                        help='Show help for this and all subscripts.')
    arguments = parser.parse_args()

    work_dir = str(pathlib.Path(__file__).parent.resolve())
    if arguments.help_all:
        subprocess.run(['python3', work_dir + '/run.py', '--help'])
        print('\n\n*** below are the help messages of the subscripts ***')
        print('\n\n*** ' + software_run_py + ' ***')
        sys.stdout.flush()
        subprocess.run(['python3', work_dir + '/' + software_run_py, '--help'])
        print('\n\n*** ' + doc_run_py + ' ***')
        sys.stdout.flush()
        subprocess.run(['python3', work_dir + '/' + doc_run_py, '--help'])
    elif arguments.software is not None:
        subprocess.run(['python3', work_dir + '/' + software_run_py] + arguments.software)
    elif arguments.doc is not None:
        subprocess.run(['python3', work_dir + '/' + doc_run_py] + arguments.doc)
    else:
        print('Unknown argument')


if __name__ == "__main__":
    main()
