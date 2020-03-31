#!/usr/bin/env python3
# This script executes all tests in `test/`.

import os
import sys
import argparse
import subprocess
import re
from pathlib import Path

cbold = "\x1b[1m"
cpass = "\x1b[32m"
cfail = "\x1b[31m"
creset = "\x1b[0m"

crate_dir = os.path.dirname(__file__)
test_dir = crate_dir + "/../test"

# Parse arguments.
parser = argparse.ArgumentParser(description="Execute all regression tests.")
parser.add_argument("--crate", metavar="DIR", default=crate_dir, help="Root directory of the llhd crate")
parser.add_argument("--debug", action="store_true", help="Use debug builds of local crate")
parser.add_argument("--release", action="store_true", help="Use release builds of local crate")
parser.add_argument("--prefix", metavar="PREFIX", help="Use binaries installed at this prefix")
parser.add_argument("--tests", metavar="DIR", default=test_dir, help="Directory where tests are located")
parser.add_argument("-v", "--verbose", action="store_true", help="Print stdout/stderr of failing tests")
args = parser.parse_args()

crate_dir = Path(os.path.realpath(args.crate))
test_dir = Path(os.path.realpath(args.tests))

# Build the binaries if requested.
prefix = None
try:
    if args.debug or args.release:
        # Build
        cmd = ["cargo", "build", "--bins"]
        if args.release:
            cmd += ["--release"]
        subprocess.run(
            cmd,
            stdin=subprocess.DEVNULL,
            cwd=crate_dir.__str__(),
            check=True,
        )

        # Extract build directory
        # cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p'
        metadata = subprocess.run(
            ["cargo", "metadata", "--format-version", "1"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            cwd=crate_dir.__str__(),
            check=True,
            universal_newlines=True,
        ).stdout
        prefix = re.search(r'"target_directory":"([^"]*)"', metadata).group(1)
        if args.debug:
            prefix += "/debug"
        if args.release:
            prefix += "/release"
        prefix = os.path.realpath(prefix)+"/"

except Exception as e:
    sys.stderr.write("{}\n".format(e))
    sys.exit(1)

prefix = prefix or args.prefix or ""
if args.verbose:
    sys.stdout.write("# crate:   {}\n".format(crate_dir))
    sys.stdout.write("# test:    {}\n".format(test_dir))
    sys.stdout.write("# prefix:  {}\n".format(prefix))

# Collect the tests.
tests = [p.relative_to(test_dir) for p in sorted(test_dir.glob("**/*.llhd"))]
sys.stdout.write("running {} tests\n".format(len(tests)))

# Execute the tests.
procs = list([
    (test, subprocess.Popen(
        ["{}llhd-check".format(prefix), os.path.realpath((test_dir / test).__str__())],
        universal_newlines=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=crate_dir.__str__(),
    )) for test in tests
])

# Output test results.
failed = list()
for test, proc in procs:
    sys.stdout.write("test {} ...".format(test))
    sys.stdout.flush()
    try:
        stdout, stderr = proc.communicate(timeout=10)
    except TimeoutExpired:
        sys.stdout.write(" timeout,")
        sys.stdout.flush()
        proc.kill()
        stdout, stderr = proc.communicate()
    if proc.returncode != 0:
        failed.append(test)
        sys.stdout.write(" {}FAILED{}\n".format(cbold+cfail, creset))
        if args.verbose:
            sys.stdout.write("\n=== STDERR ===\n")
            sys.stdout.write(stderr)
            sys.stdout.write("\n=== STDOUT ===\n")
            sys.stdout.write(stdout)
            sys.stdout.write("\n")
    else:
        sys.stdout.write(" {}passed{}\n".format(cpass, creset))

# Output summary.
sys.stdout.write("\n")
if failed:
    sys.stdout.write("failures:\n")
    for f in failed:
        sys.stdout.write("    {}\n".format(f))
    sys.stdout.write("\n")
sys.stdout.write("test result: {}. {} passed, {} failed\n".format(
    "{}PASSED{}".format(cbold+cpass, creset) if not failed else "{}FAILED{}".format(cbold+cfail, creset),
    len(tests) - len(failed),
    len(failed))
)

sys.exit(1 if failed else 0)
