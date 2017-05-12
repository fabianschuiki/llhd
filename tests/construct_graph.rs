// Copyright (c) 2017 Fabian Schuiki
extern crate llhd;

use llhd::visit::Visitor;
use llhd::inst::*;
use llhd::block::*;

#[test]
fn simple_func() {
	let module = llhd::Module::new();
	let func_ty = llhd::func_ty(vec![llhd::int_ty(32), llhd::int_ty(32)], llhd::void_ty());
	let mut func = module.add_function("foo", func_ty);

	func.args_mut()[0].set_name("a");
	func.args_mut()[1].set_name("b");
	let a = func.arg(0);
	let b = func.arg(1);

	let bb = func.add_block(Block::new(Some("entry")));

	let inst = func.add_inst(Inst::new(Some("x"), BinaryInst(BinaryOp::Add, llhd::int_ty(32), a.into(), b.into())));
	func.append_inst(inst, bb);

	let stdout = std::io::stdout();
	llhd::assembly::Writer::new(&mut stdout.lock()).visit_function(&func);
}
