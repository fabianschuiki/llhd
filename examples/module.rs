// Copyright (c) 2017-2019 Fabian Schuiki
use llhd::{int_ty, ir::prelude::*, signal_ty};

fn main() {
    let mut md = Module::new();

    // Declare an external function.
    let mut func_sig = Signature::new();
    func_sig.add_input(int_ty(32));
    func_sig.set_return_type(int_ty(3));
    md.declare(UnitName::global("my_func"), func_sig.clone());

    // Declare an external process.
    let mut proc_sig = Signature::new();
    proc_sig.add_input(signal_ty(int_ty(42)));
    proc_sig.add_output(signal_ty(int_ty(9)));
    md.declare(UnitName::global("my_proc"), proc_sig.clone());

    // Create a process which calls the external function.
    let mut proc2 = Process::new(UnitName::local("foo"), Signature::new());
    {
        let mut builder = ProcessBuilder::new(&mut proc2);
        let bb = builder.block();
        builder.append_to(bb);
        let ext = builder.add_extern(UnitName::global("my_func"), func_sig);
        let v1 = builder.ins().const_int(32, false, 9001);
        builder.ins().call(ext, vec![v1]);
    }
    md.add_process(proc2);

    // Create an entity which instantiates the processes.
    let mut ent = Entity::new(UnitName::local("bar"), Signature::new());
    {
        let mut builder = EntityBuilder::new(&mut ent);
        let ext0 = builder.add_extern(UnitName::global("my_proc"), proc_sig);
        let ext1 = builder.add_extern(UnitName::local("foo"), Signature::new());
        builder.ins().inst(ext0, vec![], vec![]);
        builder.ins().inst(ext1, vec![], vec![]);
    }
    md.add_entity(ent);

    md.link();
    println!("{}", md.dump());
}
