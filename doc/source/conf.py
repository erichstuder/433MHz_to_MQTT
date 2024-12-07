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
    'sphinx_needs',
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
html_static_path = ['_static']

html_css_files = [
    'gherkin.css',
]

drawio_no_sandbox = True

needs_types = [
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
    subprocess.run(["sphinx-gherkindoc", "../features", "source/auto_generated/features"], check=True)

def setup(app):
    app.connect("builder-inited", run_gherkindoc)
