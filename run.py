#!/usr/bin/env python3

from project_management.dispatcher import run_dispatcher

if __name__ == "__main__":
    scripts = {
        'features': 'features/run.py',
        'software': '.devcontainer/software/run.py',
        'doc': 'doc/run.py',
    }

    run_dispatcher(scripts)
