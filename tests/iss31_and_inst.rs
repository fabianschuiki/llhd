#[macro_use]
extern crate indoc;

#[test]
fn check() {
    let input = indoc! {"
        entity @foo () () {
            %0 = and i1 42 9001
        }
    "};
    llhd::assembly::parse_str(input).unwrap();
}
