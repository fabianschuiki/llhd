-- Copyright (c) 2014 Fabian Schuiki
library ieee;
use ieee.std_logic_1164.all;


-- A simple priority-based arbiter for bus multiplexing. Multiple nodes may
-- request access, the node with the lowest line is granted access until it
-- releases its request line.
entity arbiter is
	generic (
		NUM_LINES : natural := 2); -- number of lines that need arbitration
	port (
		clk_ci : in std_logic;
		request_si : in std_logic_vector(NUM_LINES-1 downto 0);  -- high indicates a node requests access
		assign_so : out std_logic_vector(NUM_LINES-1 downto 0)); -- high indicates a node is given access
end arbiter;


architecture behavioural of arbiter is
	signal assignment_dp, assignment_dn : integer range 0 to NUM_LINES := NUM_LINES;
begin

	-- Set the assign_so line high which corresponds to the current value in
	-- assignment_d.
	assignments : for i in 0 to NUM_LINES-1 generate
	begin
		assign_so(i) <= '1' when assignment_dp = i else '0';
	end generate;

	-- Assignment mechanism.
	assignment_pc : process (assignment_dp, request_si)
	begin
		if assignment_dp = NUM_LINES or (assignment_dp < NUM_LINES and request_si(assignment_dp) = '0') then
			assignment_dn <= NUM_LINES; -- default to no selection
			for i in NUM_LINES-1 downto 0 loop
				if request_si(i) = '1' then
					assignment_dn <= i;
				end if;
			end loop;
		else
			assignment_dn <= assignment_dp;
		end if;
	end process assignment_pc;

	assignment_ps : process (clk_ci)
	begin
		if clk_ci'event and clk_ci = '1' then
			assignment_dp <= assignment_dn;
		end if;
	end process assignment_ps;

end architecture;
