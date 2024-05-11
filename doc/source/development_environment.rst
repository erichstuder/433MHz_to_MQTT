Development Environment
=======================

Note: It makes no sense to have direct links to files in the project.

#. The documentation shall stand for it self.
#. There shall be no broken links, which is not possible if the links shall work locally and relativ to the online repository.

Tool Chain
----------
.. drawio-image:: tool_chain.drawio

Textfiles can also be modified with another editor.

.. collapse:: Configuration of VS Code (.vscode/)
   :open:

   .. literalinclude:: ../../.vscode/extensions.json
      :caption: extensions.json
      :language: json

   .. literalinclude:: ../../.vscode/settings.json
      :caption: settings.json
      :language: json

.. collapse:: Project File Structure
   :open:

   .. program-output:: tree -a --gitignore -I .git -F --filesfirst ../..
