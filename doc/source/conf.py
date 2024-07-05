# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import subprocess
from sphinx.application import Sphinx

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
]

rust_crates = {
    'firmware': '../software/firmware/src',
    'app': '../software/app/src',
}
rust_doc_dir = 'source/auto_generated'
rust_rustdocgen = '/root/.cargo/bin/sphinx-rustdocgen'

templates_path = ['_templates']
exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'

drawio_no_sandbox = True


def run_gherkindoc(app: Sphinx):
    subprocess.run(["sphinx-gherkindoc", "../features", "source/auto_generated/features"], check=True)

def setup(app):
    app.connect("builder-inited", run_gherkindoc)
