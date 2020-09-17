use indoc::indoc;

fn main() {
    let input = indoc! {"
        declare @bar (i32, i9) i32

        func @foo (i32 %x, i8 %y) i32 {
        %entry:
            %asdf0 = const i32 42
            %1 = const time 1.489ns 10d 9e
            %hello = alias i32 %asdf0
            %2 = not i32 %asdf0
            %3 = neg i32 %2
            %4 = add i32 %2, %3
            %5 = sub i32 %2, %3
            %6 = and i32 %2, %3
            %7 = or i32 %2, %3
            %8 = xor i32 %2, %3
            %cmp = eq i32 %7, %7
            br %cmp, %entry, %next
        %next:
            %a = exts i9, i32 %7, 4, 9
            %b = neg i9 %a
            %r = call i32 @bar (i32 %8, i9 %b)
            %many = [32 x i9 %b]
            %some = exts [9 x i9], [32 x i9] %many, 2, 9
            %one = extf i9, [9 x i9] %some, 3
            neg i9 %one
            ret i32 %3
        }

        entity @magic (i32$ %data, i1$ %clk) -> (i32$ %out) {
            %datap = prb i32$ %data
            %cmp = const i1 0
            reg i32$ %out, [%datap, rise %cmp]
        }
    "};
    println!("Dump:");
    let module = llhd::assembly::parse_module(input).unwrap();
    println!("{}", module.dump());
    println!("");
    println!("Written:");
    llhd::assembly::write_module(std::io::stdout(), &module);
}
