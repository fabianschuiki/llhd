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
	Parser.cpp
	Parser-first.cpp
	Parser-second.cpp
	Parser-subprogs.cpp
	Parser-names.cpp)
add_dependencies(llhd-vhdl vhdl-tblgen-keywords) # causes keyword table to be built before llhd-vhdl
target_link_libraries(llhd-vhdl llhd-base)

# Add the test executables.
add_executable(test-vhdl-lexer test/lexer.cpp)
add_executable(test-vhdl-parser test/parser.cpp)
target_link_libraries(test-vhdl-lexer llhd-vhdl)
target_link_libraries(test-vhdl-parser llhd-vhdl)

# Collect the test files.
file(GLOB_RECURSE lexer_files RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} test/lexer/*.vhd)
file(GLOB_RECURSE parser_files RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} test/parser/*.vhd)

foreach(f ${lexer_files})
	get_filename_component(n ${f} NAME)
	add_test(lexer-${n} ${CMAKE_CURRENT_BINARY_DIR}/test-vhdl-lexer ${CMAKE_CURRENT_SOURCE_DIR}/${f})
endforeach()
foreach(f ${parser_files})
	get_filename_component(n ${f} NAME)
	add_test(lexer-${n} ${CMAKE_CURRENT_BINARY_DIR}/test-vhdl-lexer ${CMAKE_CURRENT_SOURCE_DIR}/${f})
	add_test(parser-${n} ${CMAKE_CURRENT_BINARY_DIR}/test-vhdl-parser ${CMAKE_CURRENT_SOURCE_DIR}/${f})
endforeach()
