import inspect
import os

def get_caller_path():
    stack = inspect.stack()
    caller_file = os.path.abspath(stack[1].filename)

    for frame in stack[2:]:
        source_caller = os.path.abspath(frame.filename)
        if source_caller != caller_file:
            return os.path.dirname(source_caller)
    return os.path.dirname(source_caller)
