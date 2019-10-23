// Copyright (c) 2017-2019 Fabian Schuiki
use indoc::indoc;

macro_rules! loopback {
    ($input:tt) => {
        compare! {$input, $input}
    };
}

macro_rules! compare {
    ($input:tt, $expected:tt) => {
        let input = indoc!($input);
        let expected = indoc!($expected);
        let module = llhd::assembly::parse_module(input).unwrap();
        let asm = llhd::assembly::write_module_string(&module);
        assert_eq!(expected, asm);
    };
}

#[test]
fn empty() {
    loopback! {"
        func @a () void {
        0:
            ret
        }

        proc @b () -> () {
        0:
            halt
        }

        entity @c () -> () {
        }
    "};
}

#[test]
fn types() {
    loopback! {"
        func @foo (i32 %v0, time %v1, i32* %v2, i32$ %v3, void %v4, {i32, i64} %v5, [9001 x i32] %v6) void {
        entry:
            ret
        }
    "};
}

#[test]
fn function() {
    loopback! {"
        func @foo (i32 %a, i32 %b) void {
        entry:
            %0 = add i32 %a, %b
            %1 = const i32 42
            %y = add i32 %0, %1
            ret
        schmentry:
            %2 = eq i32 %y, %a
            ret
        }

        func @bar (i32 %0) void {
        well:
            ret
        }
    "};
}

#[test]
fn process() {
    loopback! {"
        proc @bar (i32 %0) -> (i32 %1) {
        entry:
            %k0 = const i32 21
            %k1 = const i32 42
            %2 = add i32 %0, %k0
            %y = add i32 %0, %k1
            halt
        }
    "};
}

#[test]
fn entity() {
    loopback! {"
        entity @top (i32$ %0) -> (i32$ %1) {
            %k0 = const i32 9000
            %k1 = const i32 42
            %p0 = prb i32$ %0
            %2 = add i32 %p0, %k0
            %y = add i32 %p0, %k1
        }
    "};
}

#[test]
fn call_and_inst() {
    loopback! {"
        func @foo (i32 %a, i32 %b) void {
        0:
            ret
        }

        proc @bar (i32 %a) -> (i32 %b) {
        entry:
            call void @foo (i32 %a, i32 %a)
            halt
        }

        entity @top (i32 %a) -> (i32 %b) {
            inst @bar (i32 %a) -> (i32 %b)
        }
    "};
}

#[test]
fn regression_underscore_names() {
    loopback! {"
        proc @four_pulses () -> () {
        0:
            halt
        }
    "};
}

#[test]
fn regression_signal_type() {
    loopback! {"
        proc @foo (i1$ %a) -> (i1$ %b) {
        0:
            halt
        }
    "};
}

#[test]
fn extract_with_pointer() {
    loopback! {"
        func @foo () void {
        entry:
            %k0 = const i32 0
            %k1 = const i16 0
            %a0 = var i32 %k0
            %a1 = exts i1*, i32* %a0, 3, 1
            %a2 = exts i2*, i32* %a0, 0, 2
            %k2 = {i32 %k0, i16 %k1}
            %b0 = var {i32, i16} %k2
            %b1 = extf i32*, {i32, i16}* %b0, 0
            %k3 = [4 x i32 %k0]
            %c0 = var [4 x i32] %k3
            %c1 = extf i32*, [4 x i32]* %c0, 2
            %c2 = exts [2 x i32]*, [4 x i32]* %c0, 1, 2
            ret
        }
    "};
}

#[test]
fn extract_with_signal() {
    loopback! {"
        proc @foo (i32$ %a0, {i32, i16}$ %b0, [4 x i32]$ %c0) -> () {
        entry:
            %a1 = exts i1$, i32$ %a0, 3, 1
            %a2 = exts i2$, i32$ %a0, 0, 2
            %b1 = extf i32$, {i32, i16}$ %b0, 0
            %c1 = extf i32$, [4 x i32]$ %c0, 2
            %c2 = exts [2 x i32]$, [4 x i32]$ %c0, 1, 2
            halt
        }
    "};
}

#[test]
fn nonuniform_array_regression() {
    // Check that the non-uniform array syntax amitted by the writter matches
    // the one accepted by the parser.
    loopback! {"
        proc @foo () -> () {
        entry:
            %0 = const i32 0
            %1 = const i32 0
            %2 = [i32 %0, %1]
            halt
        }
    "};
}

#[test]
fn shift_regression() {
    // Check that the parser accepts shifts properly.
    loopback! {"
        func @foo () void {
        entry:
            %0 = const i32 0
            %1 = shl i32 %0, i32 %0, i32 %0
            %2 = shr i32 %0, i32 %0, i32 %0
            ret
        }
    "};
}

#[test]
fn mux_regression() {
    // Check that the parser accepts muxes properly.
    loopback! {"
        func @foo () void {
        entry:
            %0 = const i32 0
            %1 = [i32 %0, %0]
            %2 = mux [2 x i32] %1, i32 %0
            ret
        }
    "};
}
