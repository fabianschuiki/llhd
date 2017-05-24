// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code, unused_imports)]

use std;
use combine::*;
use combine::char::{alpha_num, digit, string, spaces, space, Spaces};
use combine::combinator::{Skip, Expected, FnParser};
use module::Module;
use function::Function;
use process::Process;
use entity::Entity;
use argument::Argument;
use visit::Visitor;
use block::{Block, BlockPosition};
use seq_body::SeqBody;
use inst::*;
use value::{ValueRef, Value};
use konst;
use assembly::Writer;
use ty::*;
use num::BigInt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;


pub fn parse_str(input: &str) -> Result<Module, String> {
	match parser(module).parse(State::new(input)) {
		Ok((m, _)) => Ok(m),
		Err(err) => Err(format!("{}", err)),
	}
}


/// Applies the inner parser `p` and skips any trailing spaces.
fn lex<P>(p: P) -> Skip<P, Spaces<P::Input>>
where P: Parser, P::Input: Stream<Item = char> {
    p.skip(spaces())
}


/// Skip whitespace and comments.
fn whitespace<I>(input: I) -> ParseResult<(), I>
where I: Stream<Item = char> {
	let comment = (token(';'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());
	skip_many(skip_many1(space()).or(comment)).parse_stream(input)
}


/// Parse the part of a name after the '@' or '%' introducing it.
fn inner_name<I>(input: I) -> ParseResult<String, I>
where I: Stream<Item = char> {
	many1(alpha_num()).parse_stream(input)
}

/// Parse a global or local name, e.g. `@foo` or `%bar` respectively.
fn name<I>(input: I) -> ParseResult<(bool, String), I>
where I: Stream<Item = char> {
	(token('%').map(|_| false).or(token('@').map(|_| true)), parser(inner_name))
		.parse_stream(input)
}

/// Parse a local name, e.g. `%bar`.
fn local_name<I>(input: I) -> ParseResult<String, I>
where I: Stream<Item = char> {
	(token('%'), parser(inner_name))
		.map(|(_,s)| s)
		.parse_stream(input)
}


/// Parse a type.
fn ty<I>(input: I) -> ParseResult<Type, I>
where I: Stream<Item = char> {
	let int = many1(digit()).map(|s: String| s.parse::<usize>().unwrap());
	choice!(
		string("void").map(|_| void_ty()),
		token('i').with(int).map(|i| int_ty(i))
	).parse_stream(input)
}


/// Parse the end of a line, with an optional comment.
fn eol<I>(input: I) -> ParseResult<(), I>
where I: Stream<Item = char> {
	let comment = (token(';'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());
	skip_many(satisfy(|c: char| c.is_whitespace() && c != '\n'))
		.skip(optional(comment))
		.skip(token('\n').map(|_| ()).or(eof()))
		.skip(parser(whitespace))
		.expected("end of line")
		.parse_stream(input)
}


/// Parse a sequence of basic blocks.
fn blocks<I>(ctx: &NameTable, input: I) -> ParseResult<Vec<(Block, Vec<Inst>)>, I>
where I: Stream<Item = char> {
	let block = parser(local_name).skip(token(':')).skip(parser(eol))
		.expected("basic block")
		.and(env_parser(ctx, insts))
		.map(|(name, insts)| (Block::new(Some(name)), insts));
	many(block).parse_stream(input)
}


/// Parse a sequence of instructions.
fn insts<I>(ctx: &NameTable, input: I) -> ParseResult<Vec<Inst>, I>
where I: Stream<Item = char> {
	let name = parser(local_name).skip(spaces()).skip(token('=')).skip(spaces());
	let inst = choice!(
		try(env_parser(ctx, unary_inst)),
		try(env_parser(ctx, binary_inst)),
		try(env_parser(ctx, compare_inst)),
		try(env_parser(ctx, call_inst)),
		try(env_parser(ctx, instance_inst))
	);
	let named_inst = try(optional(name)).and(inst).skip(parser(eol)).map(|(name, inst)| {
		let inst = Inst::new(name.clone(), inst);
		if let Some(name) = name {
			ctx.insert(NameKey(false, name), inst.as_ref().into(), inst.ty());
		}
		inst
	});
	many(named_inst).parse_stream(input)
}


/// Parse a unary instruction.
fn unary_inst<I>(ctx: &NameTable, input: I) -> ParseResult<InstKind, I>
where I: Stream<Item = char> {
	let unary_op = choice!(
		string("not").map(|_| UnaryOp::Not)
	);

	// Parse the operator and type.
	let ((op, ty), consumed) = lex(unary_op).and(lex(parser(ty))).parse_stream(input)?;

	// Parse the operand, passing in the type as context.
	let (arg, consumed) = consumed.combine(|input|
		env_parser((ctx, &ty), inline_value).parse_stream(input)
	)?;

	Ok((InstKind::UnaryInst(op, ty, arg), consumed))
}


/// Parse a binary instruction.
fn binary_inst<I>(ctx: &NameTable, input: I) -> ParseResult<InstKind, I>
where I: Stream<Item = char> {
	let binary_op = choice!(
		string("add").map(|_| BinaryOp::Add),
		string("add").map(|_| BinaryOp::Add),
		string("sub").map(|_| BinaryOp::Sub),
		string("mul").map(|_| BinaryOp::Mul),
		string("div").map(|_| BinaryOp::Div),
		string("mod").map(|_| BinaryOp::Mod),
		string("rem").map(|_| BinaryOp::Rem),
		string("shl").map(|_| BinaryOp::Shl),
		string("shr").map(|_| BinaryOp::Shr),
		string("and").map(|_| BinaryOp::And),
		string("or") .map(|_| BinaryOp::Or),
		string("xor").map(|_| BinaryOp::Xor)
	);

	// Parse the operator and type.
	let ((op, ty), consumed) = lex(binary_op).and(lex(parser(ty))).parse_stream(input)?;

	// Parse the left and right hand side, passing in the type as context.
	let ((lhs, rhs), consumed) = consumed.combine(|input| (
		lex(env_parser((ctx, &ty), inline_value)),
		env_parser((ctx, &ty), inline_value)
	).parse_stream(input))?;

	Ok((InstKind::BinaryInst(op, ty, lhs, rhs), consumed))
}


/// Parse a compare instruction.
fn compare_inst<I>(ctx: &NameTable, input: I) -> ParseResult<InstKind, I>
where I: Stream<Item = char> {
	let compare_op = choice!(
		string("eq").map(|_| CompareOp::Eq),
		string("neq").map(|_| CompareOp::Neq),
		string("slt").map(|_| CompareOp::Slt),
		string("sgt").map(|_| CompareOp::Sgt),
		string("sle").map(|_| CompareOp::Sle),
		string("sge").map(|_| CompareOp::Sge),
		string("ult").map(|_| CompareOp::Ult),
		string("ugt").map(|_| CompareOp::Ugt),
		string("ule").map(|_| CompareOp::Ule),
		string("uge").map(|_| CompareOp::Uge)
	);

	// Parse the operator and type.
	let ((op, ty), consumed) = lex(string("cmp")).with(lex(compare_op)).and(lex(parser(ty))).parse_stream(input)?;

	// Parse the left and right hand side, passing in the type as context.
	let ((lhs, rhs), consumed) = consumed.combine(|input| (
		lex(env_parser((ctx, &ty), inline_value)),
		env_parser((ctx, &ty), inline_value)
	).parse_stream(input))?;

	Ok((InstKind::CompareInst(op, ty, lhs, rhs), consumed))
}


/// Parse a call instruction.
fn call_inst<I>(ctx: &NameTable, input: I) -> ParseResult<InstKind, I>
where I: Stream<Item = char> {
	let ((global, name), consumed) = lex(string("call")).with(lex(parser(name))).parse_stream(input)?;
	let (target, ty) = ctx.lookup(&NameKey(global, name));
	let (args, consumed) = {
		let mut arg_tys = ty.as_func().0.into_iter();
		let (args, consumed) = consumed.combine(|input| between(
			lex(token('(')),
			token(')'),
			sep_by(
				parser(|input| {
					env_parser((ctx, arg_tys.next().expect("missing argument")), inline_value)
					.parse_stream(input)
				}),
				lex(token(','))
			),
		).parse_stream(input))?;
		(args, consumed)
	};
	Ok(((InstKind::CallInst(ty, target, args)), consumed))
}


/// Parse an instance instruction.
fn instance_inst<I>(ctx: &NameTable, input: I) -> ParseResult<InstKind, I>
where I: Stream<Item = char> {
	let ((global, name), consumed) = lex(string("inst")).with(lex(parser(name))).parse_stream(input)?;
	let (target, ty) = ctx.lookup(&NameKey(global, name));
	let (ins, outs, consumed) = {
		let (in_tys, out_tys) = ty.as_entity();

		let mut arg_tys = in_tys.into_iter();
		let (ins, consumed) = consumed.combine(|input| between(
			lex(token('(')),
			lex(token(')')),
			sep_by(
				parser(|input| {
					env_parser((ctx, arg_tys.next().expect("missing argument")), inline_value)
					.parse_stream(input)
				}),
				lex(token(','))
			),
		).parse_stream(input))?;

		let mut arg_tys = out_tys.into_iter();
		let (outs, consumed) = consumed.combine(|input| between(
			lex(token('(')),
			token(')'),
			sep_by(
				parser(|input| {
					env_parser((ctx, arg_tys.next().expect("missing argument")), inline_value)
					.parse_stream(input)
				}),
				lex(token(','))
			),
		).parse_stream(input))?;

		(ins, outs, consumed)
	};
	Ok(((InstKind::InstanceInst(ty, target, ins, outs)), consumed))
}


/// Parse an inline value.
fn inline_value<I>((ctx, ty): (&NameTable, &Type), input: I) -> ParseResult<ValueRef, I>
where I: Stream<Item = char> {
	let const_int = (
		optional(token('-')),
		many1(digit()).map(|s: String| BigInt::parse_bytes(s.as_bytes(), 10).unwrap())
	).map(|(sign, value)| match sign {
		Some(_) => -value,
		None => value
	});

	choice!(
		parser(name).map(|(g,s)| ctx.lookup(&NameKey(g,s)).0),
		const_int.map(|value| konst::const_int(ty.as_int(), value).into())
	).parse_stream(input)
}


/// Parse a list of arguments in parenthesis.
fn arguments<I>(input: I) -> ParseResult<Vec<(Type, Option<String>)>, I>
where I: Stream<Item = char> {
	between(
		lex(token('(')),
		token(')'),
		sep_by(
			parser(ty).skip(spaces()).and(optional(parser(local_name))),
			lex(token(','))
		),
	).parse_stream(input)
}


/// Parse a function.
fn function<I>(ctx: &NameTable, input: I) -> ParseResult<Function, I>
where I: Stream<Item = char> {

	// Parse the function header.
	let (((global, name), args, return_ty), consumed) = lex(string("func")).with((
		lex(parser(name)),
		lex(parser(arguments)),
		lex(parser(ty)),
	)).parse_stream(input)?;

	// Construct the function type.
	let mut arg_tys = Vec::new();
	let mut arg_names = Vec::new();
	for (ty, name) in args {
		arg_tys.push(ty);
		arg_names.push(name);
	}
	let func_ty = func_ty(arg_tys, return_ty);

	// Construct the function and assign names to the arguments.
	let mut func = Function::new(name.clone(), func_ty.clone());
	ctx.insert(NameKey(global, name), func.as_ref().into(), func_ty);
	let ctx = &NameTable::new(Some(ctx));
	for (name, arg) in arg_names.into_iter().zip(func.args_mut().into_iter()) {
		if let Some(name) = name {
			ctx.insert(NameKey(false, name.clone()), arg.as_ref().into(), arg.ty());
			arg.set_name(name);
		}
	}

	// Parse the function body.
	let (_, consumed) = consumed.combine(|input| parse_body(ctx, input, func.body_mut()))?;

	Ok((func, consumed))
}


/// Parse a process.
fn process<I>(ctx: &NameTable, input: I) -> ParseResult<Process, I>
where I: Stream<Item = char> {

	// Parse the process header.
	let ((global, name, proc_ty, in_names, out_names), consumed) = parse_header(input, "proc")?;

	// Construct the process and assign names to the arguments.
	let mut prok = Process::new(name.clone(), proc_ty.clone());
	ctx.insert(NameKey(global, name), prok.as_ref().into(), proc_ty);
	let ctx = &NameTable::new(Some(ctx));
	let assign_names = |names: Vec<Option<String>>, args: &mut [Argument]|{
		for (name, arg) in names.into_iter().zip(args.into_iter()) {
			if let Some(name) = name {
				ctx.insert(NameKey(false, name.clone()), arg.as_ref().into(), arg.ty());
				arg.set_name(name);
			}
		}
	};
	assign_names(in_names, prok.inputs_mut());
	assign_names(out_names, prok.outputs_mut());

	// Parse the process body.
	let (_, consumed) = consumed.combine(|input| parse_body(ctx, input, prok.body_mut()))?;

	Ok((prok, consumed))
}


/// Parse an entity.
fn entity<I>(ctx: &NameTable, input: I) -> ParseResult<Entity, I>
where I: Stream<Item = char> {

	// Parse the entity header.
	let ((global, name, entity_ty, in_names, out_names), consumed) = parse_header(input, "entity")?;

	// Construct the entity and assign names to the arguments.
	let mut entity = Entity::new(name.clone(), entity_ty.clone());
	ctx.insert(NameKey(global, name), entity.as_ref().into(), entity_ty);
	let ctx = &NameTable::new(Some(ctx));
	let assign_names = |names: Vec<Option<String>>, args: &mut [Argument]|{
		for (name, arg) in names.into_iter().zip(args.into_iter()) {
			if let Some(name) = name {
				ctx.insert(NameKey(false, name.clone()), arg.as_ref().into(), arg.ty());
				arg.set_name(name);
			}
		}
	};
	assign_names(in_names, entity.inputs_mut());
	assign_names(out_names, entity.outputs_mut());

	// Parse the entity body.
	let (insts, consumed) = consumed.combine(|input| between(
		token('{').skip(parser(eol)),
		token('}').skip(parser(eol)),
		env_parser(ctx, insts),
	).parse_stream(input))?;
	for inst in insts {
		entity.add_inst(inst, InstPosition::End);
	}

	Ok((entity, consumed))
}


/// Parse the body of a function or process.
fn parse_body<I>(ctx: &NameTable, input: I, body: &mut SeqBody) -> ParseResult<(), I>
where I: Stream<Item = char> {
	let (blocks, consumed) = between(
		token('{').skip(parser(eol)),
		token('}').skip(parser(eol)),
		env_parser(ctx, blocks),
	).parse_stream(input)?;

	for (block, insts) in blocks {
		let bb = body.add_block(block, BlockPosition::End);
		for inst in insts {
			body.add_inst(inst, InstPosition::BlockEnd(bb));
		}
	}

	Ok(((), consumed))
}


/// Parse the header of a process or entity.
fn parse_header<I>(input: I, keyword: &'static str) -> ParseResult<(bool, String, Type, Vec<Option<String>>, Vec<Option<String>>), I>
where I: Stream<Item = char> {

	// Parse the header.
	let (((global, name), ins, outs), consumed) = lex(string(keyword)).with((
		lex(parser(name)),
		lex(parser(arguments)),
		lex(parser(arguments)),
	)).parse_stream(input)?;

	// Construct the type.
	let split = |args|{
		let mut arg_tys = Vec::new();
		let mut arg_names = Vec::new();
		for (ty, name) in args {
			arg_tys.push(ty);
			arg_names.push(name);
		}
		(arg_tys, arg_names)
	};
	let (in_tys, in_names) = split(ins);
	let (out_tys, out_names) = split(outs);
	let unit_ty = entity_ty(in_tys, out_tys);

	Ok(((global, name, unit_ty, in_names, out_names), consumed))
}


/// Parse a module.
fn module<I>(input: I) -> ParseResult<Module, I>
where I: Stream<Item = char> {
	let mut module = Module::new();
	let tbl = NameTable::new(None);

	enum Thing {
		Function(Function),
		Process(Process),
		Entity(Entity),
	}

	let thing = choice!(
		env_parser(&tbl, function).map(|f| Thing::Function(f)),
		env_parser(&tbl, process).map(|p| Thing::Process(p)),
		env_parser(&tbl, entity).map(|e| Thing::Entity(e))
	);

	(parser(whitespace), many::<Vec<_>, _>(thing), eof())
		.parse_stream(input)
		.map(|((_, things, _),r)|{
			for thing in things {
				match thing {
					Thing::Function(f) => { module.add_function(f); },
					Thing::Process(p) => { module.add_process(p); },
					Thing::Entity(e) => { module.add_entity(e); },
				}
			}
			(module,r)
		})
}


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NameKey(bool, String);

struct NameTable<'tp>(Option<&'tp NameTable<'tp>>, Rc<RefCell<HashMap<NameKey, (ValueRef, Type)>>>);

impl<'tp> NameTable<'tp> {
	/// Create a new name table with an optional parent.
	pub fn new(parent: Option<&'tp NameTable<'tp>>) -> NameTable<'tp> {
		NameTable(parent, Rc::new(RefCell::new(HashMap::new())))
	}

	/// Insert a name into the table.
	pub fn insert(&self, key: NameKey, value: ValueRef, ty: Type) {
		let mut map = self.1.borrow_mut();
		if map.insert(key, (value, ty)).is_some() {
			panic!("name redefined");
		}
	}

	/// Lookup a name in the table.
	pub fn lookup(&self, key: &NameKey) -> (ValueRef, Type) {
		if let Some(v) = self.1.borrow().get(key) {
			return v.clone();
		}
		if let Some(p) = self.0 {
			return p.lookup(key);
		}
		panic!("name {}{} has not been declared", if key.0 { "@" } else { "%" }, key.1);
	}
}


#[test]
fn parse_test() {
	let text = r#"
func @foo (i32 %a, i32 %b) void {
%entry:
    %0 = add i32 %a %b
    %y = add i32 %0 42
%schmentry:
    %1 = cmp eq i32 %y %a
}

func @bar (i32) void {
%well:
}
	"#;
	let (module, _) = parser(module).parse(State::new(text)).unwrap();
	let stdout = std::io::stdout();
	Writer::new(&mut stdout.lock()).visit_module(&module);
}
