Development Environment
=======================

The development environment uses **VS Code** as the editor.
While this documentation only describes the use of VS Code, the project may be developed with any editor.

To maintain toolchains and dependencies, **Docker** is used.

**Python** is used as the scripting language, e.g. to interact with Docker.

Installation
------------
To work on this project on a local machine, the following tools need to be installed:

- VS Code
- Docker
- Python

For Debugging there might be some more tools necessary. (see: :ref:`Debugging`)

VS Code
-------
All necessary files can be created and edited with VS Code.

.. drawio-image:: vs_code_files.drawio

configuration
^^^^^^^^^^^^^
.. collapse:: configuration files of VS Code are found in the .vscode/ directory

   .. literalinclude:: ../../.vscode/extensions.json
      :caption: extensions.json
      :language: json

   .. literalinclude:: ../../.vscode/settings.json
      :caption: settings.json
      :language: json

.gitignore(s)
^^^^^^^^^^^^^
There might be multiple .gitignore files.
They are placed with reasonable granularity (documentation, test, code, ...).
See :doc:`file_structure` where they are.

reStructuredText
^^^^^^^^^^^^^^^^
The documentation is written with Sphinx and reStructuredText.
It may contain diagrams, which are created with draw.io and stuff from other sources.
There may also be PlantUML diagrams directly embedded into code documentation or other documentation files.

draw.io files
^^^^^^^^^^^^^
Diagrams especially for the documentation are created with draw.io.

workflows
^^^^^^^^^
There shall be as much automation as reasonable possible.
GitHub Actions with workflows are used for automation.

build scripts
^^^^^^^^^^^^^
For simple project handling (test, build, download to target, ...) Python scripts are used.
As the same scripts are used by the GitHub Actions workflows, the build process is consistent locally and remotely.
The scripts are very powerfull and almost everything can be done with them.
See see: :ref:`Scripts` how they work.

Dockerfile(s)
^^^^^^^^^^^^^
There might be multiple Dockerfiles.
They are placed with reasonable granularity (documentation, test, code, ...).
See :doc:`file_structure` where they are.
Everything generating output like compilers, flash-tools, unit-test-frameworks,
documentation-build-chain, ... are in the Docker containers.
This makes the development environment consistent, reproducible and documented.

code
^^^^
Rust has been chosen as the programming language, as it is an upcoming language with a lot of potential and a good community.
It might not be the easiest solution for the task, but that is not a criterium here.
See :doc:`file_structure` for the code structure.


Local Toolchain
---------------

.. drawio-image:: toolchain_local.drawio


Remote Toolchain
----------------

.. drawio-image:: toolchain_remote.drawio

Git
^^^
Is used for version control.

github.com
^^^^^^^^^^
GitHub is used as the repository host.

Python
^^^^^^
Python is used to execute the scripts.

Docker
^^^^^^
Runs the containers.

Sphinx
^^^^^^
Is used to generate the documentation.

Rust tools
^^^^^^^^^^
The Rust tools are used to build and test the code and to deploy to the target.

actions
^^^^^^^
GitHub Actions are used to automate the build and test process.


.. _Debugging:

Debugging
---------
This chapter explains how to setup the debugging for the project.

**This chapter is subject to change.
For example some tools might be moved to the docker container in the future.
And more documentation is to come.
TODOs:**

- **Document Wiring**
- **Move stuff into Container?**
- **Explain how exactly to setup the tools for debugging.**
- **remove user specific paths form launch.json**
- **describe how to debug using unit tests**


Other debugging setup might work as well but are not tested.

.. plantuml::

   @startuml

   [Computer] <-> [SEGGER J-Link EDU Mini]
   [SEGGER J-Link EDU Mini] <-> [RPi Pico W]

   @enduml

There are two plugins configured for direct debugging in VS Code:

- `Debugger for probe-rs <https://marketplace.visualstudio.com/items?itemName=probe-rs.probe-rs-debugger>`_
- `Cortex-Debug <https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug>`_

Both of them are set up in the launch.json file.
See the documentation of the them what to install and how to use them.

Another very powerfull tool proven to work is `Ozone <https://www.segger.com/products/development-tools/ozone-j-link-debugger/>`_.

launch.json
^^^^^^^^^^^^
.. program-output:: cat ../../software/.vscode/launch.json


.. _Scripts:

Scripts
-------

The scripts help to simplify development workflows like creating documentation,
running tests, flashing firmware and more.
They are very powerfull and are in most cases the way to interact with the
development environment if you want to execute any task.

There is a main script run.py in the project root folder.
All the scripts in subfolders can be run from this script.
The scripts in the subfolders are also named run.py.
So any run.py belongs to the script system described here.
Any run.py can be run from the folder it is in.
Additionally the main script in the project root folder can run all of them.

The scripts are written in Python as this is widely available are easy to read
and simplify the parsing of command line arguments.
Bash scripts for example have been found to be less readable and less flexible.

Note: The automated build system uses exactly the same scripts, what leads to even more consistency.

run.py (project root)
^^^^^^^^^^^^^^^^^^^^^
This is the main script in the root folder. Everything can be done from here.
It acts as a wrapper for the other scripts and forwards the parameters to them.

For example to run the tests, you can use the following command.
This forwards the `-t` parameter to the software script.

`python3 run.py -s -t`

The local script can be run in exactly the same way but without the `-s` parameter.

See the help message for more information.

help message
^^^^^^^^^^^^

To get this help message run in the project root folder:

:code:`python3 run.py --help_all`

.. program-output:: python3 ../../run.py --help_all
