// Copyright (c) 2017-2019 Fabian Schuiki
use llhd::ir::prelude::*;

fn main() {
    let name = UnitName::global("foo");
    let sig = Signature::new();
    let mut ent = UnitData::new(UnitKind::Entity, name, sig);
    {
        let mut builder = UnitBuilder::new_anonymous(&mut ent);
        let mut sig = Signature::new();
        sig.add_input(llhd::signal_ty(llhd::int_ty(1)));
        sig.add_input(llhd::signal_ty(llhd::int_ty(42)));
        sig.add_output(llhd::signal_ty(llhd::int_ty(32)));
        let ext = builder.add_extern(UnitName::global("bar"), sig);
        let v1 = builder.ins().const_int((1, 0));
        let v2 = builder.ins().const_int((42, 9001));
        let v3 = builder.ins().const_int((32, 0));
        let v4 = builder.ins().sig(v3);
        builder.ins().inst(ext, vec![v1, v2], vec![v4]);
    }
    println!("{}", Unit::new_anonymous(&ent));
}
