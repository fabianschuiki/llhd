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
