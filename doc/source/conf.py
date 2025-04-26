# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import subprocess
from sphinx.application import Sphinx
import os

project = '433MHz_to_MQTT'
copyright = '2024, erichstuder'
author = 'erichstuder'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinxcontrib.drawio',
    'sphinxcontrib.plantuml',
    'sphinx_toolbox.collapse',
    'sphinxcontrib.programoutput',
    'sphinxcontrib_rust',
    'sphinx_needs',
]

rust_crates = {
    'firmware': '../software/firmware',
}
rust_doc_dir = 'source/auto_generated'
rust_visibility = 'pvt'
rust_rustdocgen = '/root/.cargo/bin/sphinx-rustdocgen'

templates_path = ['_templates']

exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']

html_css_files = [
    'gherkin.css',
]

drawio_no_sandbox = True

needs_types = [
    {
        "directive": "feature",
        "title": "Feature",
        "prefix": "F_",
        "color": "#BFD8D2",
        "style": "node",
    },
    {
        "directive": "usecase",
        "title": "Use Case",
        "prefix": "UC_",
        "color": "#BFD8D2",
        "style": "usecase",
    },
    {
        "directive": "actor",
        "title": "Actor",
        "prefix": "A_",
        "color": "#BFD8D2",
        "style": "actor",
    },
]

needs_extra_links = [
    {
        "option": "includes",
        "incoming": "is included by",
        "outgoing": "<<include>>",
        "copy": False,
        "style": "#000000",
        "style_part": "#000000",
        "style_start": ".",
        "style_end": "->"
    },
    {
        "option": "association",
        "incoming": "is associated with",
        "outgoing": "",
        "copy": False,
        "style": "#000000",
        "style_part": "#000000",
        "style_start": "-",
        "style_end": "-"
    },
]

def run_gherkindoc(app: Sphinx):
    features_dir = os.path.join(app.srcdir, 'auto_generated/features')
    subprocess.run(['sphinx-gherkindoc', '--raw-descriptions', '--doc-project', 'DOC_PROJECT', '../features', features_dir], check=True)
    subprocess.run(['rm', os.path.join(features_dir, 'gherkin.rst')], check=True) # Prevent unused sphinx file warning.

    # Remove the '%' character from the beginning of lines in files in the source/auto_generated/features directory
    # This is a workaround for now as the parser removes all whitespaces from the beginning of lines which leads to invalid requirements.
    for root, _, files in os.walk(features_dir):
        for file in files:
            file_path = os.path.join(root, file)
            with open(file_path, 'r') as f:
                lines = f.readlines()
            with open(file_path, 'w') as f:
                for line in lines:
                    if line.startswith('%'):
                        f.write(line[1:])  # Remove the '%' character
                    else:
                        f.write(line)

def run_cargo_modules(app: Sphinx):
    software_dependencies_path = os.path.join(app.srcdir, 'auto_generated/software_dependencies.png')
    cargo_process = subprocess.Popen(['cargo', 'modules', 'dependencies', '--manifest-path', '../software/firmware',
                      '--no-externs', '--no-fns', '--no-owns', '--no-traits', '--no-types'], stdout=subprocess.PIPE)
    subprocess.run(['dot', '-Tpng', '-o', software_dependencies_path], stdin=cargo_process.stdout, check=True)

def setup(app: Sphinx):
    app.connect("builder-inited", run_gherkindoc)
    app.connect("builder-inited", run_cargo_modules)
