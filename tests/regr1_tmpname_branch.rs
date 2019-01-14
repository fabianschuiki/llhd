#[macro_use]
extern crate indoc;

#[test]
fn check_parsed() {
    let input = indoc! {"
        proc @foo () () {
        %0:
            br label %0
        }
    "};
    let module = llhd::assembly::parse_str(input).unwrap();
    assert_eq!(
        llhd::assembly::write_string(&module),
        indoc! {"
            proc @foo () () {
            %0:
                br label %0
            }
        "}
    );
}

#[test]
fn check_constructed() {
    let mut module = llhd::Module::new();
    let ty = llhd::entity_ty(vec![], vec![]);
    let mut prok = llhd::Process::new("foo".to_string(), ty.clone());
    let blk = prok
        .body_mut()
        .add_block(llhd::Block::new(None), llhd::BlockPosition::End);
    prok.body_mut().add_inst(
        llhd::Inst::new(None, llhd::BranchInst(llhd::BranchKind::Uncond(blk))),
        llhd::InstPosition::End,
    );
    module.add_process(prok);
    assert_eq!(
        llhd::assembly::write_string(&module),
        indoc! {"
            proc @foo () () {
            %0:
                br label %0
            }
        "}
    );
}
