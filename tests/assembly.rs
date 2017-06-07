// Copyright (c) 2017 Fabian Schuiki

extern crate llhd;


struct LineTrimIter<I: Iterator<Item = char>>(I, bool);

impl<I: Iterator<Item = char>> Iterator for LineTrimIter<I> {
	type Item = I::Item;

	fn next(&mut self) -> Option<I::Item> {
		let mut c = self.0.next();
		while self.1 && c.map(|d| d == '\t').unwrap_or(false) {
			c = self.0.next();
		}
		self.1 = c == Some('\n');
		c
	}
}

fn parse(input: &str) -> llhd::Module {
	llhd::assembly::parse_str(input).unwrap()
}

fn loopback(input: &str) {
	use llhd::visit::Visitor;
	let mut v = Vec::new();
	llhd::assembly::Writer::new(&mut v).visit_module(&parse(input));
	let a = String::from_utf8(v).unwrap();
	let e: String = LineTrimIter(input.chars().skip(1), true).collect();
	assert_eq!(e, a);
}


#[test]
fn empty() {
	loopback(r#"
		func @a () void {
		}

		proc @b () () {
		}

		entity @c () () {
		}
	"#);
}

#[test]
fn function() {
	loopback(r#"
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
	"#);
}

#[test]
fn process() {
	loopback(r#"
		proc @bar (i32 %0) (i32 %1) {
		%entry:
		    %2 = add i32 %0 21
		    %y = add i32 %0 42
		}
	"#);
}

#[test]
fn entity() {
	loopback(r#"
		entity @top (i32 %0) (i32 %1) {
		    %2 = add i32 %0 9000
		    %y = add i32 %0 42
		}
	"#);
}

#[test]
fn call_and_inst() {
	loopback(r#"
		func @foo (i32 %a, i32 %b) void {
		}

		proc @bar (i32 %a) (i32 %b) {
		%entry:
		    call @foo (%a, %a)
		}

		entity @top (i32 %a) (i32 %b) {
		    inst @bar (%a) (%b)
		}
	"#);
}

#[test]
fn instructions() {
	loopback(r#"
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
	"#);
}

#[test]
fn regression_underscore_names() {
	parse(r#"
		proc @four_pulses () () {
		}
	"#);
}

#[test]
fn regression_signal_type() {
	parse(r#"
		proc @foo (i1$ %a) (i1$ %b) {
		}
	"#);
}
