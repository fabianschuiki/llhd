#[macro_use]
extern crate indoc;

#[test]
fn check() {
    let input = indoc! {"
        entity @foo.bar () () {
        }
    "};
    llhd::assembly::parse_str(input).unwrap();
}
