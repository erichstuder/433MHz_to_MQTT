# You must run this script without sudo. To run docker without sudo do the following:
# sudo groupadd docker
# sudo gpasswd -a $USER docker

import argparse
import subprocess
import datetime
import pathlib

def parse_arguments():
    parser = argparse.ArgumentParser(description='Execute common documentation tasks')

    parser.add_argument('--build', '-b',
                        action='store_true',
                        help='Build documentation.')

    parser.add_argument('--sphinx_autobuild', '--sa',
                        action='store_true',
                        help='Start sphinx-autobuild.')

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

    args.append(work_dir)

    print('    building container... please wait')
    return subprocess.run(args, stdout=stdout)


def run_container(container_tag, work_dir):

    current_time = datetime.datetime.now().strftime('%Hh_%Mm_%Ss');

    docker_volume_dir = '/usr/project'

    prebuild_command = 'git config --global --add safe.directory ' + docker_volume_dir

    work_dir_commands = 'set -e \n cd doc \n'

    if arguments.keep_open:
        commands = 'bash'
    elif arguments.sphinx_autobuild:
        commands = work_dir_commands + 'sphinx-autobuild '+ ('' if arguments.verbose else '-q') +' -a --port 8000 --host 0.0.0.0 '
        commands += '--watch ../software/firmware/src --watch ../software/app/src --re-ignore auto_generated source _build/html '
        commands += '--pre-build "' + prebuild_command + '"'
        print(commands)
    elif arguments.build:
        commands = work_dir_commands + prebuild_command + ' \n make html'
    else:
        return

    args = ['docker', 'run',
        '--rm',
        '--name', 'doc_' + current_time,
        '--volume', work_dir + '/..:' + docker_volume_dir,
        '--workdir', docker_volume_dir]

    if arguments.sphinx_autobuild:
        args.extend([
            '--publish', '8000:8000',
            '--publish', '35729:35729'])

    if arguments.pseudo_tty_disable:
        args.append('-i')
    else:
        args.append('-it')

    args.extend([container_tag, 'bash', '-c', commands])

    return subprocess.run(args)


def assert_result(result):
    if result is not None and result.returncode != 0:
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
