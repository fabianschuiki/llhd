// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables, unused_mut)]

extern crate llhd;
extern crate num;

use llhd::visit::Visitor;
use llhd::inst::*;
use llhd::block::*;

#[test]
fn simple_func() {
	let mut module = llhd::Module::new();

	let func_ty = llhd::func_ty(vec![llhd::int_ty(32), llhd::int_ty(32)], llhd::void_ty());
	let mut func = llhd::Function::new("foo".into(), func_ty.clone());
	{
		func.args_mut()[0].set_name("a");
		func.args_mut()[1].set_name("b");
		let a = func.arg(0);
		let b = func.arg(1);
		let body = func.body_mut();

		let bb = body.add_block(Block::new(Some("entry".into())), BlockPosition::End);

		let inst = body.add_inst(Inst::new(None, BinaryInst(BinaryOp::Add, llhd::int_ty(32), a.into(), b.into())), InstPosition::End);
		let konst = llhd::const_int(32, 42.into());
		let inst = body.add_inst(Inst::new(Some("y".into()), BinaryInst(BinaryOp::Add, llhd::int_ty(32), inst.into(), konst.into())), InstPosition::End);
	}
	let func = module.add_function(func);

	let proc_ty = llhd::entity_ty(vec![llhd::int_ty(32)], vec![llhd::int_ty(32)]);
	let mut prok = llhd::Process::new("bar".into(), proc_ty.clone());
	{
		let a = prok.input(0);
		let body = prok.body_mut();
		body.add_block(Block::new(Some("entry".into())), BlockPosition::End);
		body.add_inst(Inst::new(None, BinaryInst(BinaryOp::Add, llhd::int_ty(32), a.into(), llhd::const_int(32, 21.into()).into())), InstPosition::End);
		body.add_inst(Inst::new(None, CallInst(func_ty.clone(), func.into(), vec![a.into(), a.into()])), InstPosition::End);
	}
	let prok = module.add_process(prok);

	let mut entity = llhd::Entity::new("top".into(), proc_ty.clone());
	{
		let a = entity.input(0);
		let b = entity.output(0);
		entity.add_inst(Inst::new(None, BinaryInst(BinaryOp::Add, llhd::int_ty(32), a.into(), llhd::const_int(32, 9000.into()).into())), InstPosition::End);
		entity.add_inst(Inst::new(None, InstanceInst(proc_ty.clone(), prok.into(), vec![a.into()], vec![b.into()])), InstPosition::End);
	}
	module.add_entity(entity);

	let stdout = std::io::stdout();
	llhd::assembly::Writer::new(&mut stdout.lock()).visit_module(&module);
}
