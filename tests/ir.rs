use llhd::ir::prelude::*;

/// Create a `func @test() void` function populated by a callback. Useful to act
/// as a simple container to collect instructions.
fn within_func(return_type: llhd::Type, f: impl FnOnce(&mut UnitBuilder)) -> UnitData {
    let name = UnitName::global("test");
    let mut sig = Signature::new();
    sig.set_return_type(return_type);
    let mut func = UnitData::new(UnitKind::Function, name, sig);
    let mut builder = UnitBuilder::new_anonymous(&mut func);
    let bb = builder.named_block("entry");
    builder.append_to(bb);
    f(&mut builder);
    func
}

#[test]
fn call_with_void() {
    within_func(llhd::void_ty(), |builder| {
        let mut sig = Signature::new();
        sig.set_return_type(llhd::void_ty());
        let ext = builder.add_extern(UnitName::global("foo"), sig);
        let result = builder.ins().name("x").call(ext, vec![]);
        println!("{}", builder.unit());
        assert_eq!(builder.value_type(result), llhd::void_ty());
    });
}

#[test]
fn call_with_return_value() {
    within_func(llhd::int_ty(32), |builder| {
        let mut sig = Signature::new();
        sig.add_input(llhd::int_ty(1));
        sig.add_input(llhd::int_ty(42));
        sig.set_return_type(llhd::int_ty(32));
        let ext = builder.add_extern(UnitName::global("foo"), sig);
        let v1 = builder.ins().name("one").const_int((1, 0));
        let v2 = builder.ins().const_int((42, 9001));
        let v3 = builder.ins().suffix(v1, "called").call(ext, vec![v1, v2]);
        builder.ins().ret_value(v3);
        println!("{}", builder.unit());
    });
}
