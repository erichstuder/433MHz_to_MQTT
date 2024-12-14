#!/usr/bin/env python3

import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent.parent / 'project_management'))
from executor import Executor # type: ignore


if __name__ == "__main__":
    additional_arguments = [
        {
            'flag': '-b',
            'name': '--build',
            'help': 'Build documentation.'
        },
        {
            'flag': '-a',
            'name': '--autobuild',
            'help': 'Start sphinx-autobuild.'
        }
    ]

    ex = Executor(additional_arguments, description='Execute unit-tests')

    if ex.arguments.build:
        commands = 'make html'
    elif ex.arguments.autobuild:
        commands = 'sphinx-autobuild '+ ('' if ex.arguments.verbose else '-q') +' --port 8000 --host 0.0.0.0 '
        commands += '--watch ../software/firmware/src --watch ../software/app/src --watch ../features '
        commands += '--re-ignore auto_generated source _build/html'
    else:
        commands = None

    ex.run(commands)
