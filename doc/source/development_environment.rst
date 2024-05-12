Development Environment
=======================

Note: It makes no sense to have direct links to files in the project.

#. The documentation shall stand for it self.
#. There shall be no broken links, which is not possible if the links shall work locally and relativ to the online repository.

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

draw.io files
^^^^^^^^^^^^^
Diagrams especially for the documentation are created with draw.io.

workflows
^^^^^^^^^
There shall be as much automation as possible.
GitHub Actions with workflows are used for this.

build scripts
^^^^^^^^^^^^^
For simple project handling (test, build, download to target, ...) Python scripts are used.
As the same scripts are used by the GitHub Actions workflows, the build process is consistent locally and remotely.

Dockerfile(s)
^^^^^^^^^^^^^
There might be multiple Dockerfiles.
They are placed with reasonable granularity (documentation, test, code, ...).
See :doc:`file_structure` where they are.
VS Code with some extensions and Docker are the only tools that need to be installed locally.
Everything generating output like compilers, flash-tools, unit-test-frameworks,
documentation-build-chain, ... are in the Docker containers.
This makes the development environment consistent and reproducible.

code
^^^^
Rust has been chosen as the programming language, as it is an upcoming language with a lot of potential and a good community.
It might not be the easiest solution for the task, but that is not a criterium here.
See :doc:`file_structure` for the code structure.
