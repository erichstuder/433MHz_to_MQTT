Development Environment
=======================

The development environment uses **VS Code** as the editor.
While this documentation only describes the use of VS Code, the project may be developed with any editor.

To maintain toolchains and dependencies, **Docker** is used.

**Python** is used as the scripting language, e.g. to interact with Docker.

Installation
------------
To work on this project on a local machine, the following tools need to be installed.
The version of the tools is uncritical, as all the critical tools and dependencies are in the Docker containers.

- `Visual Studio Code (VS Code) <https://code.visualstudio.com/>`_
- `Docker <https://www.docker.com/>`_
- `Docker Compose <https://docs.docker.com/compose/>`_
- `Python <https://www.python.org/>`_

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

.. collapse:: devcontainer definitions are found in the .devcontainer/ directory

   .. literalinclude:: ../../.devcontainer/doc/devcontainer.json
      :caption: devcontainer.json
      :language: json

   .. literalinclude:: ../../.devcontainer/features/devcontainer.json
      :caption: devcontainer.json
      :language: json

   .. literalinclude:: ../../.devcontainer/software/devcontainer.json
      :caption: devcontainer.json
      :language: json

.. raw:: html

   <br>

Devcontainers are Docker containers that VS Code can open into.
It guarantiees a consistent development environment while using the benefits of VS Code.
See for more information on `devcontainers <https://code.visualstudio.com/docs/devcontainers/containers>`_.

.gitignore(s)
^^^^^^^^^^^^^
There might be multiple .gitignore files.
They are placed with reasonable granularity.
See :doc:`file_structure` where they are.

reStructuredText
^^^^^^^^^^^^^^^^
The documentation is created with Sphinx from reStructuredText.
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
The scripts are powerfull and almost everything can be done with them.
See see: :ref:`Scripts` how they work.

Dockerfile(s)
^^^^^^^^^^^^^
There are multiple Dockerfiles and docker-compose.yml files.
They are placed with reasonable granularity.
See :doc:`file_structure` where they are.
Everything generating output like compilers, flash-tools, unit-test-frameworks,
documentation-build-chain, ... are in the Docker containers.
Docker Compose files control the configuration of the containers (e.g. ports, volumes, ...).
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
Together with Docker Compose, defines the containers.

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
This chapter explains how to setup the debugging with SEGGER J-Link EDU Mini and rs-probe.

.. plantuml::

   @startuml
   [Devcontainer with \n probe-rs] <-> [VS Code]
   [VS Code] <-> [Development Workstation]
   [Development Workstation] <-(0->  [SEGGER J-Link EDU Mini] : "  USB"
   [SEGGER J-Link EDU Mini] <-(0-> [RPi Pico W] : "  SWD"
   @enduml



SEGGER J-Link EDU Mini
^^^^^^^^^^^^^^^^^^^^^^
The SEGGER J-Link EDU Mini is used as a debugger.
As this is not a commercial project its use is permited.
Other SEGGER debuggers might work in the same way.

The easiest way to make the debugger work under Linux is to install `J-Link Software and Documentation Pack <https://www.segger.com/downloads/jlink/#J-LinkSoftwareAndDocumentationPack>`_.
This will automatically set the necessary udev rules.

On Windows it might just work.

probe-rs
^^^^^^^^
probe-rs is installed in the container and the plugin is used in the devcontainer.
So, no installation by hand.

.. collapse:: Dockerfile with installation of probe-rs

   .. literalinclude:: ../../software/Dockerfile
      :caption: $/software/Dockerfile

.. collapse:: devcontainer with extension probe-rs.probe-rs-debugger

   .. literalinclude:: ../../.devcontainer/software/devcontainer.json
      :caption: $/.devcontainer/software/devontainer.json

.. raw:: html

   <br>

other setups
^^^^^^^^^^^^

Other debugging setup might work as well but are not tested.

`Cortex-Debug <https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug>`_ can also work with VS Code.

Another very powerful tool **without** VS Code is `Ozone <https://www.segger.com/products/development-tools/ozone-j-link-debugger/>`_.

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

The scripts are written in Python as this is widely available, spripts are easy to read
and simplifies the parsing of command line arguments.
Bash scripts for example have been found to be less readable and less flexible.

The github workflows use the exact same scripts, what leads to even more consistency.

Whenever you feel lost there is help in every script.

**./run.py \--help**

run.py (project root)
^^^^^^^^^^^^^^^^^^^^^
This is the main script in the root folder. Everything can be done from here.
It acts as a wrapper for the other scripts and forwards the parameters to them.

For example to run unit-tests, you can use the following command.

*./run.py \--software \--test*

This forwards the *\--test* parameter to the software script.
The local script can be run in the same way but without the *\--software* parameter.

See the help message for more information.

help message
^^^^^^^^^^^^

To get this help message run in the project root folder:

:code:`./run.py --help_all`

.. program-output:: ./../../run.py --help_all
