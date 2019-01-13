#[macro_use]
extern crate indoc;

use llhd::Value;

#[test]
fn check() {
    let input = indoc! {"
        entity @Foo () () {
            %0 = sig i32
        }
    "};
    let module = llhd::assembly::parse_str(input).unwrap();
    let ent = module.entity(module.values().next().unwrap().unwrap_entity());
    for inst in ent.insts() {
        assert_eq!(inst.name(), None);
    }
}
