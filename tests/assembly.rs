// Copyright (c) 2017 Fabian Schuiki

#[macro_use]
extern crate indoc;

use llhd::Visitor;

macro_rules! loopback {
    ($input:tt) => {
        compare! {$input, $input}
    };
}

macro_rules! compare {
    ($input:tt, $expected:tt) => {
        let input = indoc!($input);
        let expected = indoc!($expected);
        let module = llhd::assembly::parse_str(input).unwrap();
        let mut asm = Vec::new();
        llhd::assembly::Writer::new(&mut asm).visit_module(&module);
        let asm = String::from_utf8(asm).unwrap();
        assert_eq!(expected, asm);
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
fn types() {
    loopback! {"
        func @foo () void {
        %entry:
            %v0 = var i32
            %v1 = var time
            %v2 = var i32*
            %v3 = var i32$
            %v4 = var void
            %v5 = var {i32, i64}
            %v6 = var [9001 x i32]
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
            %v0 = var i8
            %v1 = load i8 %v0
            store i8 %v0 42
            %i1 = insert element i32 %v0, 0, i32 42
            %i2 = insert slice i8 %v0, 0, 2, i2 3
            %e1 = extract element i32 %v0, 0
            %e2 = extract slice i8 %v0, 0, 2
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

#[test]
fn extract_with_pointer() {
    loopback! {"
        func @foo () void {
        %entry:
            %a0 = var i32
            %a1 = extract element i32* %a0, 3
            %a2 = extract slice i32* %a0, 0, 2
            %b0 = var {i32, i16}
            %b1 = extract element {i32, i16}* %b0, 0
            %c0 = var [4 x i32]
            %c1 = extract element [4 x i32]* %c0, 2
            %c2 = extract slice [4 x i32]$ %c0, 1, 2
        }
    "};
}

#[test]
fn extract_with_signal() {
    loopback! {"
        proc @foo (i32$ %a0, {i32, i16}$ %b0, [4 x i32]$ %c0) () {
        %entry:
            %a1 = extract element i32* %a0, 3
            %a2 = extract slice i32* %a0, 0, 2
            %b1 = extract element {i32, i16}* %b0, 0
            %c1 = extract element [4 x i32]* %c0, 2
            %c2 = extract slice [4 x i32]$ %c0, 1, 2
        }
    "};
}
