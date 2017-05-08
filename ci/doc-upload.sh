#!/bin/bash
# Copyright (c) 2017 Fabian Schuiki
# This script uploads the documentation in target/doc to the repo's
# GitHub pages. Requires the GH_TOKEN env var to be set.

set -e

COMMIT=`git rev-parse --short HEAD`
MSG="Documentation for llhd ($COMMIT)"

echo "<meta http-equiv=refresh content=0;url=llhd/index.html>" > target/doc/index.html

[ -d ghp-import ] || git clone https://github.com/davisp/ghp-import
ghp-import/ghp_import.py -n -m "$MSG" target/doc
git push -fq https://${GH_TOKEN}@github.com/fabianschuiki/llhd.git gh-pages
