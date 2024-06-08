import argparse
import subprocess
import pathlib
import sys

scripts = {
    'features': 'features/run.py',
    'software': 'software/run.py',
    'doc': 'doc/run.py',
}


def main():
    parser = argparse.ArgumentParser(description='Execute common tasks like building, testing, uploading, ...')
    group = parser.add_mutually_exclusive_group(required=True)
    for key, value in scripts.items():
        group.add_argument('--'+key, '-'+key[0],
                           nargs=argparse.REMAINDER,
                           help='Pass the remaining arguments to ' + value + '.')
    group.add_argument('--help_all', '--ha',
                       action='store_true',
                       help='Show help for this and all subscripts.')
    arguments = parser.parse_args()

    work_dir = str(pathlib.Path(__file__).parent.resolve())
    if arguments.help_all:
        subprocess.run(['python3', work_dir + '/run.py', '--help'])
        print('\n\n*** below are the help messages of the subscripts ***')
        for _, value in scripts.items():
            print('\n\n*** ' + value + ' ***')
            sys.stdout.flush()
            subprocess.run(['python3', work_dir + '/' + value, '--help'])
    elif arguments.features is not None:
        subprocess.run(['python3', work_dir + '/' + scripts['features']] + arguments.features)
    elif arguments.software is not None:
        subprocess.run(['python3', work_dir + '/' + scripts['software']] + arguments.software)
    elif arguments.doc is not None:
        subprocess.run(['python3', work_dir + '/' + scripts['doc']] + arguments.doc)
    else:
        print('Unknown argument')


if __name__ == "__main__":
    main()
