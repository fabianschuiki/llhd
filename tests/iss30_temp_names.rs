#[macro_use]
extern crate indoc;

use llhd::Value;

#[test]
fn check() {
    let input = indoc! {"
        entity @Foo (void %0) (void %1) {
            %2 = sig i32
        }
    "};
    let module = llhd::assembly::parse_str(input).unwrap();
    let ent = module.entity(module.values().next().unwrap().unwrap_entity());
    for arg in ent.inputs() {
        assert_eq!(arg.name(), None);
    }
    for arg in ent.outputs() {
        assert_eq!(arg.name(), None);
    }
    for inst in ent.insts() {
        assert_eq!(inst.name(), None);
    }
}
