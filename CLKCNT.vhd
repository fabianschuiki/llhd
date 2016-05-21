-- A testbench that generates a clock signal and produces some output.
entity CLKCNT is
end entity;

architecture tb of CLKCNT is
	signal CLK : std_logic;
	signal CNT : unsigned(7 downto 0) := (others => '0');
begin
	clkgen: process begin
		CLK <= '0';
		wait for 0.5 ns;
		CLK <= '1';
		wait for 0.5 ns;
	end process;

	counter: process (CLK) begin
		if CLK'event and CLK = '1' then
			CNT <= CNT + 1;
		end if;
	end process;
end architecture;


-- *** EQUIVALENT LLHD ***
--
-- entity @CLKCNT () () {
--     %CLK = sig i1
--     %CNT = sig i8
--     inst comp()(i1$) @clkgen () (%CLK)
--     inst comp(i1$, i8$)(i8$) @counter (%CLK, %CNT) (%CNT)
-- }
--
-- proc @clkgen () (i1$ %CLK) {
-- entry:
--     drv i1$ %CLK 0
--     wait label %high, 0.5ns
-- high:
--     drv i1$ %CLK 1
--     wait label %entry, 0.5ns
-- }
--
-- proc @counter (i1$ %CLK, i8$ %CNT) (i8$ %CNT0) {
-- entry:
--     %CLKprev = var i1
--     %7 = prb i1$ %CLK
--     store i1* %CLKprev %7
-- sense:
--     %CLKnow = prb i1$ %CLK
--     %3 = load i1* %CLKprev
--     store i1* %CLKprev %CLKnow
--     %0 = cmp ne i1 %CLKnow %CLKprev
--     %1 = cmp eq i1 %CLKnow 1
--     %2 = and i1 %0 %1
--     br i1 %2 label %update, label %wait
-- wait:
--     wait label %sense, %CLK
-- update:
--     %4 = prb i8$ %CNT
--     %5 = add i8 %4 1
--     drv i8$ %CNT0 %5
--     br label %wait
-- }
