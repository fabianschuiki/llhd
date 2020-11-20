#!/usr/bin/env python3
# This script executes all tests in `test/`.

import os
import sys
import argparse
import subprocess
import re
import shlex
from pathlib import Path
from copy import copy
import itertools

cbold = "\x1b[1m"
cpass = "\x1b[32m"
ctime = "\x1b[33m"
cfail = "\x1b[31m"
creset = "\x1b[0m"

crate_dir = os.path.dirname(__file__) + "/.."
test_dir = crate_dir + "/tests"

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
parser.add_argument("--check", metavar="FILE", nargs=2, help="Check first file against directives in second file")
parser.add_argument("--check-stdin", metavar="FILE", nargs=1, help="Check stdin against directives in file")
parser.add_argument("TEST", nargs="*", help="Specific test cases to run; all if omitted")
args = parser.parse_args()

# A class to encapsulate the check directives in a file.
class CheckFile:
    regex_dir = re.compile(r'^\s*;\s*(CHECK[^:]*):\s+(.+)$')
    ansi_escape = re.compile(r'(?:\x1B[@-_]|[\x80-\x9F])[0-?]*[ -/]*[@-~]')

    def __init__(self, checks, input):
        # Collect the directives in the file.
        self.dirs = list()
        for line in checks.splitlines():
            m = self.regex_dir.match(line)
            if m:
                self.dirs.append((m.group(1), m.group(2)))

        # Split input into lines.
        self.input = input.splitlines()

    def execute(self):
        self.failed = list()
        self.state = iter(self.input)

        # Execute the directives in order.
        for d in self.dirs:
            try:
                self.state = self.execute_directive(d, copy(self.state))
            except Exception as e:
                try:
                    line = next(copy(self.state)).strip()
                except StopIteration:
                    line = "<end of file>"
                self.failed.append((d, e.__str__(), line))

        # Concatenate the failures into information messages.
        info = ""
        for f in self.failed:
            info += "error: {}: {}\n  {}. Scanning from:\n  {}\n".format(f[0][0], f[0][1], f[1], f[2])

        # Package a return value.
        return (len(self.failed) == 0, info)

    def execute_directive(self, directive, state):
        dirname = directive[0].replace("CHECK-ERR", "CHECK")
        if dirname == "CHECK":
            for line in state:
                line = line.split(";")[0].strip()
                line = self.ansi_escape.sub("", line)
                if line == directive[1]:
                    return state
            raise Exception("No matching line found")
        elif dirname == "CHECK-NEXT":
            try:
                line = next(state)
                line = line.split(";")[0].strip()
                line = self.ansi_escape.sub("", line)
                if line == directive[1]:
                    return state
            finally:
                pass
            raise Exception("Next line does not match")
        else:
            raise Exception("Unknown directive `{}`".format(directive[0]))


# Handle the special case where we are just supposed to check a file.
check = None
if args.check:
    with open(args.check[0]) as f:
        content = f.read()
    check = (args.check[1], content)
if args.check_stdin:
    content = sys.stdin.read()
    check = (args.check_stdin, content)
if check is not None:
    with open(check[0]) as f:
        checks = f.read()
    passed, info = CheckFile(checks, check[1]).execute()
    if passed:
        sys.exit(0)
    else:
        sys.stdout.write(info)
        sys.exit(1)

# Establish some working directories.
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
            subprocess.check_call(
                cmd,
                stdin=subprocess.DEVNULL,
                cwd=crate_dir.__str__(),
            )

        # Extract build directory
        # cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p'
        metadata = subprocess.check_output(
            ["cargo", "metadata", "--format-version", "1"],
            stdin=subprocess.DEVNULL,
            cwd=crate_dir.__str__(),
            universal_newlines=True,
        )
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
    regex_ignore = re.compile(r'^\s*;\s*IGNORE\b', flags=re.MULTILINE)
    regex_fail   = re.compile(r'^\s*;\s*FAIL\b', flags=re.MULTILINE)
    regex_run    = re.compile(r'^\s*;\s*RUN:\s+(.+)$', flags=re.MULTILINE)

    def __init__(self, name, path):
        self.name = name
        self.path = Path(path)
        self.dir = self.path.parent

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

        # Execution results.
        self.timeout = False
        self.failed = False
        self.info = ""
        self.stdout = ""
        self.stderr = ""

        # Process the run command.
        self.cmd = shlex.split(self.run)
        if self.cmd[0].startswith("llhd-"):
            self.cmd[0] = "{}{}".format(prefix, self.cmd[0])
        self.cmd = [path if x == "%s" else x for x in self.cmd]
        cmd = list()
        for x in self.cmd:
            if "*" in x.__str__():
                self.info += "Arg: `{}` is a glob pattern\n".format(x)
                cmd += self.dir.glob(x)
            elif (self.dir/x).exists():
                self.info += "Arg: `{}` is a path\n".format(x)
                cmd.append(self.dir/x)
            else:
                self.info += "Arg: `{}` is a plain argument\n".format(x)
                cmd.append(x)
        self.cmd = list([x.__str__() for x in cmd])

    def launch(self):
        if self.ignore:
            return
        try:
            self.info += "Command: {}\n".format(self.cmd)
            self.proc = subprocess.Popen(
                [x.__str__() for x in self.cmd],
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
            self.proc.kill()
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

        # Perform the file checks.
        passed, info = CheckFile(self.content, self.stdout + self.stderr).execute()
        if not passed:
            self.failed = True
            self.info += "File checks failed:\n\n"
            self.info += info

# Collect the tests.
if args.TEST:
    tests = [TestCase(p, Path(os.path.realpath(p))) for p in args.TEST]
else:
    suffices = ["llhd"]
    third_party = test_dir/"third-party"
    globs = [[p for p in test_dir.glob("**/*."+suffix) if third_party not in p.parents] for suffix in suffices]
    tests = [TestCase(p.relative_to(test_dir), p) for p in sorted(itertools.chain(*globs))]
sys.stdout.write("running {} tests\n".format(len(tests)))

# Assemble the test arrays.
num_parallel = 16
heads = tests + [None]*num_parallel
tails = [None]*num_parallel + tests

# Execute the tests and output test results.
ignored = list()
failed = list()
for head, tail in zip(heads, tails):
    if head:
        head.launch()
    if tail:
        sys.stdout.write("test {} ...".format(tail.name))
        sys.stdout.flush()
        tail.finish()
        if tail.ignore:
            ignored.append(tail)
            sys.stdout.write(" ignored\n")
            continue
        if tail.timeout:
            sys.stdout.write(" timeout,")
        if tail.failed:
            failed.append(tail)
            sys.stdout.write(" {}FAILED{}\n".format(cbold+cfail, creset))
            if args.verbose:
                sys.stdout.write("\n=== INFO ===\n")
                sys.stdout.write(tail.info)
                sys.stdout.write("\n=== STDERR ===\n")
                sys.stdout.write(tail.stderr)
                sys.stdout.write("\n=== STDOUT ===\n")
                sys.stdout.write(tail.stdout)
                sys.stdout.write("\n")
        else:
            sys.stdout.write(" {}passed{}\n".format(cpass, creset))
        if args.commands:
            sys.stdout.write("# {}\n".format(" ".join([x.__str__() for x in tail.cmd])))

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
