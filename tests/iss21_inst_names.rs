#[macro_use]
extern crate indoc;
use llhd::Visitor;

#[test]
fn check() {
    let mut module = llhd::Module::new();
    let ty = llhd::entity_ty(vec![], vec![]);

    let ent1 = llhd::Entity::new("Foo".to_string(), ty.clone());
    let ent1 = module.add_entity(ent1);

    let mut ent2 = llhd::Entity::new("Bar".to_string(), ty.clone());
    ent2.add_inst(
        llhd::Inst::new(
            Some("baz".to_string()),
            llhd::InstKind::InstanceInst(ty.clone(), ent1.into(), vec![], vec![]),
        ),
        llhd::InstPosition::End,
    );
    module.add_entity(ent2);

    let mut asm = vec![];
    llhd::assembly::Writer::new(&mut asm).visit_module(&module);
    let asm = String::from_utf8(asm).unwrap();

    assert_eq!(
        asm,
        indoc! {"
        entity @Foo () () {
        }

        entity @Bar () () {
            %baz = inst @Foo () ()
        }
        "}
    );
}
