-- A positive edge triggered clock gate latch.
entity LAGCE is
	port (
		CK  : in  std_logic;
		E   : in  std_logic;
		GCK : out std_logic
	);
end entity;

architecture rtl of LAGCE is
	signal Q : std_logic;
begin
	process (CK,E) begin
		if (CK = '0') then
			Q <= E;
			GCK <= '0';
		else
			GCK <= Q;
		end if;
	end process;
end architecture;


-- *** EQUIVALENT LLHD ***
--
-- entity @LAGCE (i1 %CK, i1 %E) (i1 %GCK) {
--     %Q = sig i1
--     inst @LAGCE_proc(%CK, %E, %Q) (%GCK, %Q)
-- }
--
-- proc @LAGCE_proc (i1 %CK, i1 %E, i1 %Qi) (i1 %GCK, i1 %Qo) {
--   entry:
--     %0 = cmp i1 %CK 0
--     br i1 %0, label %ckl, label %ckh
--   ckl:
--     drv i1 %Qo %E
--     drv i1 %GCK 0
--     ret
--   ckh:
--     drv i1 %GCK %Qi
--     ret
-- }


-- *** ALTERNATIVE LLHD ***
-- This approach requires that signals be defineable within processes and that
-- each process only runs once, with wait statements inserted to provide
-- reaction to events.
--
-- proc @LAGCE (i1 %CK, i1 %E) (i1 %GCK) {
--   entry:
--     %Q = sig i1
--   sense:
--     %0 = cmp i1 %CK 0
--     br i1 %0, label %ckl, label %ckh
--   ckl:
--     drv i1 %Q %E
--     drv i1 %GCK 0
--     br label %cont
--   ckh:
--     drv i1 %GCK %Q
--     br label %cont
--   cont:
--     wait label %sense
-- }
