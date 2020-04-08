#!/usr/bin/env python3
# This script executes all tests in `test/`.

import os
import sys
import argparse
import subprocess
import re
import shlex
from pathlib import Path

cbold = "\x1b[1m"
cpass = "\x1b[32m"
ctime = "\x1b[33m"
cfail = "\x1b[31m"
creset = "\x1b[0m"

crate_dir = os.path.dirname(__file__) + "/.."
test_dir = crate_dir + "/test"

# Parse arguments.
parser = argparse.ArgumentParser(description="Execute all regression tests.")
parser.add_argument("--crate", metavar="DIR", default=crate_dir, help="Root directory of the llhd crate")
parser.add_argument("--debug", action="store_true", help="Use debug builds of local crate")
parser.add_argument("--release", action="store_true", help="Use release builds of local crate")
parser.add_argument("--prefix", metavar="PREFIX", help="Use binaries installed at this prefix")
parser.add_argument("--tests", metavar="DIR", default=test_dir, help="Directory where tests are located")
parser.add_argument("-v", "--verbose", action="store_true", help="Print stdout/stderr of failing tests")
parser.add_argument("-c", "--commands", action="store_true", help="Print commands used to execute tests")
parser.add_argument("-n", "--no-build", action="store_true", help="Don't rebuild the project")
parser.add_argument("TEST", nargs="*", help="Specific test cases to run; all if omitted")
args = parser.parse_args()

crate_dir = Path(os.path.realpath(args.crate))
test_dir = Path(os.path.realpath(args.tests))

# Build the binaries if requested.
prefix = None
try:
    if args.debug or args.release:
        # Build
        if not args.no_build:
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

# A class to encapsulate the execution and checking of a single test.
class TestCase(object):
    regex_ignore = re.compile(r'^;\s*IGNORE\b', flags=re.MULTILINE)
    regex_fail   = re.compile(r'^;\s*FAIL\b', flags=re.MULTILINE)
    regex_run    = re.compile(r'^;\s*RUN:\s+(.+)$', flags=re.MULTILINE)

    def __init__(self, name, path):
        self.name = name
        self.path = path

        # Load the contents of the test file.
        with open(path.__str__()) as f:
            self.content = f.read()

        # Check for a `IGNORE` directive.
        self.ignore = self.regex_ignore.search(self.content) is not None

        # Check for a `FAIL` directive.
        self.should_fail = self.regex_fail.search(self.content) is not None

        # Check for a `RUN` directive.
        self.run = self.regex_run.search(self.content)
        if self.run:
            self.run = self.run.group(1)
        else:
            self.run = "llhd-check %s"

        # Process the run command.
        self.cmd = shlex.split(self.run)
        if self.cmd[0].startswith("llhd-"):
            self.cmd[0] = "{}{}".format(prefix, self.cmd[0])
        self.cmd = [path if x == "%s" else x for x in self.cmd]

        # Execution results.
        self.timeout = False
        self.failed = False
        self.info = ""
        self.stdout = ""
        self.stderr = ""

    def launch(self):
        if self.ignore:
            return
        try:
            self.info += "Command: {}\n".format(self.cmd)
            self.proc = subprocess.Popen(
                self.cmd,
                universal_newlines=True,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd=crate_dir.__str__(),
            )
        except Exception as e:
            self.info += "Exception: {}\n".format(e)
            self.failed = True

    def finish(self):
        if self.ignore or self.failed:
            return
        try:
            self.stdout, self.stderr = self.proc.communicate(timeout=10)
        except subprocess.TimeoutExpired as e:
            proc.kill()
            self.stdout, self.stderr = self.proc.communicate()
            self.timeout = True
            self.failed = True
            self.info += "Timeout"
            return

        # Check the return code.
        self.failed = (self.proc.returncode != 0)

        # Check if we were supposed to fail, but didn't.
        if self.failed != self.should_fail:
            self.info += "Failed: {}\nShould Fail: {}\n".format(self.failed, self.should_fail)
            self.failed = True
            return
        else:
            self.failed = False

# Collect the tests.
if args.TEST:
    tests = [TestCase(p, Path(os.path.realpath(p))) for p in args.TEST]
else:
    tests = [TestCase(p.relative_to(test_dir), p) for p in sorted(test_dir.glob("**/*.llhd"))]
sys.stdout.write("running {} tests\n".format(len(tests)))

# Execute the tests.
for test in tests:
    test.launch()

# Output test results.
ignored = list()
failed = list()
for test in tests:
    sys.stdout.write("test {} ...".format(test.name))
    sys.stdout.flush()
    test.finish()
    if test.ignore:
        ignored.append(test)
        sys.stdout.write(" ignored\n")
        continue
    if test.timeout:
        sys.stdout.write(" timeout,")
    if test.failed:
        failed.append(test)
        sys.stdout.write(" {}FAILED{}\n".format(cbold+cfail, creset))
        if args.verbose:
            sys.stdout.write("\n=== INFO ===\n")
            sys.stdout.write(test.info)
            sys.stdout.write("\n=== STDERR ===\n")
            sys.stdout.write(test.stderr)
            sys.stdout.write("\n=== STDOUT ===\n")
            sys.stdout.write(test.stdout)
            sys.stdout.write("\n")
    else:
        sys.stdout.write(" {}passed{}\n".format(cpass, creset))
    if args.commands:
        sys.stdout.write("# {}\n".format(" ".join([x.__str__() for x in test.cmd])))

# Output summary.
sys.stdout.write("\n")
if failed:
    sys.stdout.write("failures:\n")
    for test in failed:
        sys.stdout.write("    {}\n".format(test.name))
    sys.stdout.write("\n")
sys.stdout.write("test result: {}. {} passed, {} failed, {} ignored\n".format(
    "{}PASSED{}".format(cbold+cpass, creset) if not failed else "{}FAILED{}".format(cbold+cfail, creset),
    len(tests) - len(failed) - len(ignored),
    len(failed),
    len(ignored)
))

sys.exit(1 if failed else 0)
