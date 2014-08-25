# Copyright (c) 2014 Fabian Schuiki
# Configures the VHDL subcomponent of LLHD.

# Generate the keyword mapping table.
add_executable(vhdl-tblgen-keywords tblgen-keywords.cpp)
add_custom_command(
	TARGET vhdl-tblgen-keywords POST_BUILD
	COMMAND vhdl-tblgen-keywords
		${CMAKE_CURRENT_SOURCE_DIR}/keywords.hpp
		> ${CMAKE_CURRENT_BINARY_DIR}/vhdl-tblgen-keywords.log
	DEPENDS vhdl-tblgen-keywords
	COMMENT "Generating VHDL keywords table")

add_library(llhd-vhdl
	Lexer.cpp
	Parser.cpp)
add_dependencies(llhd-vhdl vhdl-tblgen-keywords) # causes keyword table to be built before llhd-vhdl
target_link_libraries(llhd-vhdl llhd-base)

# Add the tests.
add_executable(test-vhdl-lexer test/lexer.cpp)
add_executable(test-vhdl-parser test/parser.cpp)
target_link_libraries(test-vhdl-lexer llhd-vhdl)
target_link_libraries(test-vhdl-parser llhd-vhdl)
add_test(vhdl-lexer-1 test-vhdl-lexer ${CMAKE_CURRENT_SOURCE_DIR}/test/arbiter.vhd)
add_test(vhdl-lexer-2 test-vhdl-lexer ${CMAKE_CURRENT_SOURCE_DIR}/test/lexer.vhd)
add_test(vhdl-lexer-3 test-vhdl-lexer ${CMAKE_CURRENT_SOURCE_DIR}/test/lexer-broken1.vhd)
# add_test(vhdl-parser-1 test-vhdl-parser ${CMAKE_CURRENT_SOURCE_DIR}/test/arbiter.vhd)
