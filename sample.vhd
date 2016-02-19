-- alu
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity alu is
port (
	a, b : in std_logic_vector(7 downto 0);
	op : in std_logic_vector(1 downto 0);
	r : out std_logic_vector(7 downto 0)
);
end alu;

architecture rtl of alu is
begin
	process (a,b,op) begin
		case op is
			when "00" => r <= std_logic_vector(signed(a) + signed(b));
			when "01" => r <= std_logic_vector(signed(a) - signed(b));
			when "10" => r <= a and b;
			when "11" => r <= a or b;
			when others => r <= (others => 'U');
		end case;
	end process;
end rtl;


-- testbench
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity alu_tb is
end alu_tb;

architecture tb of alu_tb is
	signal a, b : std_logic_vector(7 downto 0);
	signal op : std_logic_vector(1 downto 0);
	signal r : std_logic_vector(7 downto 0);
begin
	dut: entity alu port map (
		a => a,
		b => b,
		op => op,
		r => r
	);

	p_stim: process begin
		a <= "00010010";
		b <= "00001010";
		op <= "00"; wait for 1 ns; -- r = "00011100"
		op <= "01"; wait for 1 ns; -- r = "00001000"
		op <= "10"; wait for 1 ns; -- r = "00000010"
		op <= "11"; wait for 1 ns; -- r = "00011010"
	end process;
end tb;
