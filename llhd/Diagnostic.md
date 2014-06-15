# Design Goals for Diagnostics

Diagnostic messages should be helpful and easy to understand. They should convey
the structure of errors and make it easy to pinpoint the context where they
appear.

The LLVM approach would look something like:

    arbiter.vhd:14-18: error: assigning to a signal with the '=' operator
      output_do = '1' when true else '0';
                ^
    fixit: use the '<=' operator to assign to signals
      output_do = '1' when true else '0';
                ~
                <=

Likely to be a better variant:

    error: assigning to signal 'output_do' with the '=' operator
      arbiter.vhd:14:
        output_do = '1' when true else '0';
                  ^
    fixit: use the '<=' operator to assign to signals
      arbiter.vhd:14:
        output_do = '1' when true else '0';
                  ~
                  <=

The latter makes sense especially when referencing other parts of the code:

    error: component declaration 'arbiter' (1) disagrees with the corresponding entity (2)
      (1) system.vhd:86-90:
        component arbiter is
                  ^^^^^^^
            port(
                output_do : out std_logic;
                error_so  : out std_logic);
        end component component_name;

      (2) arbiter.vhd:6-10:
        entity arbiter is
               ~~~~~~~
            port(
                error_so  : out std_logic;
                output_do : out std_logic);
        end entity arbiter;

    note: both declare the same port signals, however their order differs:
      (1) declares
        - output_do
        - error_so
      (2) declares
        - error_so
        - output_do

    fixit: assuming the entity declaration (2) is authorative:
      system.vhd:88:
        output_do : out std_logic;
        ~~~~~~~~~
        error_so

      system.vhd:89:
        error_so  : out std_logic);
        ~~~~~~~~~
        output_do
