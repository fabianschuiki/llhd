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


# Define a custom pygments lexer for LLHD.
import re
from pygments.lexer import RegexLexer, words
from pygments.token import Keyword, Name, Text, Comment, Operator, Number, Punctuation
from sphinx.highlighting import lexers

class LLHDLexer(RegexLexer):
	name = 'llhd'
	filenames = ['*.llhd']
	flags = re.MULTILINE

	tokens = {
		'root': [
			(r'\n', Text),
			(r'\s+', Text),
			(r';.*?$', Comment.Single),
			(r'[=]', Operator),
			(r'[()\[\]{},.]', Punctuation),
			(r'[%@]([0-9a-zA-Z_.$]|\\[0-9a-fA-F]{2})+', Name),
			(r'([0-9a-zA-Z_.$]|\\[0-9a-fA-F]{2})+:', Name.Label),
			(r'[ilsn][0-9]+|void|metadata|label|time|\$|\*', Keyword.Type),
			(words("const type func proc entity decl call inst wait br drive probe add sub mul div mod rem cmp alloc free var sig not and or xor now ret laod store".split(), suffix=r'\b'), Keyword),
			(r'[0-9]+(\.[0-9]+)?([pnum]?s)', Number.Time),
			(r'[0-9]+', Number.Integer),
			(r'[a-zA-Z_]', Text),
		]
	}

lexers['llhd'] = LLHDLexer()
