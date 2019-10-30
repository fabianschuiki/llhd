// Copyright (c) 2017-2019 Fabian Schuiki
use llhd::ir::prelude::*;

fn main() {
    let name = UnitName::global("foo");
    let mut sig = Signature::new();
    sig.set_return_type(llhd::int_ty(32));
    let mut func = Function::new(name, sig);
    {
        let mut builder = FunctionBuilder::new(&mut func);
        let bb = builder.block();
        builder.append_to(bb);
        let mut sig = Signature::new();
        sig.add_input(llhd::int_ty(1));
        sig.add_input(llhd::int_ty(42));
        sig.set_return_type(llhd::int_ty(32));
        let ext = builder.add_extern(UnitName::global("bar"), sig);
        let v1 = builder.ins().name("one").const_int(1, 0);
        let v2 = builder.ins().const_int(42, 9001);
        let v3 = builder.ins().suffix(v1, "called").call(ext, vec![v1, v2]);
        let v3 = builder.dfg().inst_result(v3);
        builder.ins().ret_value(v3);
    }
    println!("{}", func.dump());
}
