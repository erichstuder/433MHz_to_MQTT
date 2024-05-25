Build Scripts
=============

These scripts help to simplify development workflows like creating documentation, running tests or flashing firmware.
The scripts can be run from the root of the repository or the folder they are in.

run.py
------
This is the main script in the root folder. Everything can be done from here.
It acts as a wrapper for the other scripts and forwards the parameters to them.

For example to run the tests, you can use the following command.
This forwards the `-t` parameter to the software script.

`python3 run.py -s -t`

The local script can be run in exactly the same way but without `-s` parameter.

help message
^^^^^^^^^^^^
.. program-output:: python3 ../../run.py --help_all
