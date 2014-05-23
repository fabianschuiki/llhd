add_library(llhd-vhdl Lexer.cpp Parser.cpp Token.cpp)

# Add the tests.
add_executable(test-vhdl-parser test/parser.cpp)
target_link_libraries(test-vhdl-parser llhd-vhdl)
add_test(vhdl-parser test-vhdl-parser ${CMAKE_CURRENT_SOURCE_DIR}/arbiter.vhd)
