#!/usr/bin/env python3
# Copyright (c) 2017 Fabian Schuiki
#
# This file contains the Sphinx configuration for the high-level LLHD
# documentation. For API documentation look into the output of rustdoc.

from recommonmark.parser import CommonMarkParser

project = 'LLHD'
copyright = '2017, Fabian Schuiki'
author = 'Fabian Schuiki'

version = '0.1.0' # short version
release = '0.1.0' # long version, including alpha/beta/rc tags

master_doc = 'index'
source_suffix = ['.rst', '.md']
source_parsers = {
    '.md': CommonMarkParser,
}

exclude_patterns = ['_build']
pygments_style = 'sphinx'
todo_include_todos = False

# HTML output
html_theme = 'sphinx_rtd_theme'
