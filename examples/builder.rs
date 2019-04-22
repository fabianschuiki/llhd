// Copyright (c) 2017-2019 Fabian Schuiki
use llhd::ir::prelude::*;
use llhd::pass::ConstantFoldingPass;

fn main() {
    let mut func = build_function(UnitName::Global("foo".to_owned()));
    let mut prok = build_process(UnitName::Global("bar".to_owned()));
    let mut ent = build_entity(UnitName::Global("top".to_owned()));
    println!("{}", func.dump());
    println!("{}", prok.dump());
    println!("{}", ent.dump());
    println!("");
    println!("Constant Folding");
    println!("");
    ConstantFoldingPass::run_on_function(&mut func);
    ConstantFoldingPass::run_on_process(&mut prok);
    ConstantFoldingPass::run_on_entity(&mut ent);
    println!("{}", func.dump());
    println!("{}", prok.dump());
    println!("{}", ent.dump());
}

fn build_function(name: UnitName) -> Function {
    let mut sig = Signature::new();
    let arg1 = sig.add_input(llhd::int_ty(32));
    let arg2 = sig.add_input(llhd::int_ty(32));
    sig.set_return_type(llhd::void_ty());
    let mut func = Function::new(name, sig);
    {
        let mut builder = FunctionBuilder::new(&mut func);
        let arg1 = builder.unit().arg_value(arg1);
        let arg2 = builder.unit().arg_value(arg2);
        let bb1 = builder.block();
        let bb2 = builder.block();
        builder.append_to(bb1);
        let v1 = builder.ins().const_int(32, 4);
        let v2 = builder.ins().const_int(32, 5);
        let v3 = builder.ins().add(v1, v2);
        let v8 = builder.ins().umul(arg1, v3);
        let v9 = builder.ins().not(v8);
        let _v9 = builder.ins().neg(v9);
        builder.ins().br(bb2);
        builder.append_to(bb2);
        let v4 = builder.ins().const_int(32, 1);
        let v5 = builder.ins().add(v3, v4);
        let v6 = builder.ins().add(v5, arg1);
        let v7 = builder.ins().add(arg2, v6);
        builder.ins().ult(v3, v4);
        builder.ins().ugt(v3, v4);
        builder.ins().ule(v3, v4);
        builder.ins().uge(v3, v4);
        builder.ins().ret_value(v7);
    }
    func.verify();
    func
}

fn build_process(name: UnitName) -> Process {
    let mut sig = Signature::new();
    let clk = sig.add_input(llhd::signal_ty(llhd::int_ty(1)));
    let inp = sig.add_input(llhd::signal_ty(llhd::int_ty(32)));
    let oup = sig.add_output(llhd::signal_ty(llhd::int_ty(32)));
    let mut prok = Process::new(name, sig);
    {
        let mut builder = ProcessBuilder::new(&mut prok);
        let clk = builder.unit().arg_value(clk);
        let inp = builder.unit().arg_value(inp);
        let _oup = builder.unit().arg_value(oup);
        let entry_bb = builder.block();
        builder.append_to(entry_bb);
        builder.ins().add(clk, inp);
        builder.ins().eq(clk, inp);
        builder.ins().neq(clk, inp);
        builder.ins().halt();
    }
    prok.verify();
    prok
}

fn build_entity(name: UnitName) -> Entity {
    let mut sig = Signature::new();
    let _clk = sig.add_input(llhd::signal_ty(llhd::int_ty(1)));
    let _rst = sig.add_input(llhd::signal_ty(llhd::int_ty(1)));
    let inp = sig.add_input(llhd::signal_ty(llhd::int_ty(32)));
    let _oup = sig.add_output(llhd::signal_ty(llhd::int_ty(32)));
    let mut ent = Entity::new(name, sig);
    {
        let mut builder = EntityBuilder::new(&mut ent);
        let v1 = builder.ins().const_int(32, 42);
        let v2 = builder.ins().const_int(32, 2);
        let v3 = builder.ins().add(v1, v2);
        let inp = builder.unit().arg_value(inp);
        builder.ins().add(v3, inp);
    }
    ent.verify();
    ent
}
