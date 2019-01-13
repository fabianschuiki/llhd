// Copyright (c) 2017 Fabian Schuiki

#[macro_use]
extern crate indoc;

use llhd::Visitor;

macro_rules! loopback {
    ($input:tt) => {
        let input = indoc!($input);
        let module = llhd::assembly::parse_str(input).unwrap();
        let mut asm = Vec::new();
        llhd::assembly::Writer::new(&mut asm).visit_module(&module);
        let asm = String::from_utf8(asm).unwrap();
        assert_eq!(input, asm);
    };
}

#[test]
fn empty() {
    loopback! {"
        func @a () void {
        }

        proc @b () () {
        }

        entity @c () () {
        }
    "};
}

#[test]
fn function() {
    loopback! {"
        func @foo (i32 %a, i32 %b) void {
        %entry:
            %0 = add i32 %a %b
            %y = add i32 %0 42
        %schmentry:
            %1 = cmp eq i32 %y %a
        }

        func @bar (i32 %0) void {
        %well:
        }
    "};
}

#[test]
fn process() {
    loopback! {"
        proc @bar (i32 %0) (i32 %1) {
        %entry:
            %2 = add i32 %0 21
            %y = add i32 %0 42
        }
    "};
}

#[test]
fn entity() {
    loopback! {"
        entity @top (i32 %0) (i32 %1) {
            %2 = add i32 %0 9000
            %y = add i32 %0 42
        }
    "};
}

#[test]
fn call_and_inst() {
    loopback! {"
        func @foo (i32 %a, i32 %b) void {
        }

        proc @bar (i32 %a) (i32 %b) {
        %entry:
            call @foo (%a, %a)
        }

        entity @top (i32 %a) (i32 %b) {
            inst @bar (%a) (%b)
        }
    "};
}

#[test]
fn instructions() {
    loopback! {"
        func @foo (i32 %a, i32 %b, time %x) void {
        %entry:
            wait %entry
            wait %entry for %x
            wait %entry, %a, %b
            wait %entry for %x, %a, %b
            ret
            ret i32 %a
            ret i32 42
            br label %entry
            br %a label %entry %entry
            %a0 = sig i8
            %a1 = sig i8 42
            %a2 = prb %a0
            drv %a0 %a
            drv %a0 42
            drv %a1 %b %x
            halt
        }
    "};
}

#[test]
fn regression_underscore_names() {
    loopback! {"
        proc @four_pulses () () {
        }
    "};
}

#[test]
fn regression_signal_type() {
    loopback! {"
        proc @foo (i1$ %a) (i1$ %b) {
        }
    "};
}
