# Copyright (c) 2014-2015 Fabian Schuiki
# Configures the various unit tests of the LLHD codebase.

if (LLHD_BUILD_LEGACY)
	set(SRC ${CMAKE_SOURCE_DIR}/llhd)

	add_executable(test-asm-lexer
		${SRC}/AssemblyLexer.cpp
		asm-lexer.cpp)

	add_executable(test-asm-parser
		${SRC}/AssemblyLexer.cpp
		${SRC}/AssemblyParser.cpp
		${SRC}/console.cpp
		${SRC}/diagnostic/DiagnosticFormatterConsole.cpp
		${SRC}/SourceLocation.cpp
		${SRC}/SourceManager.cpp
		${SRC}/SourceManagerEntry.cpp
		asm-parser.cpp)

	target_link_libraries(test-asm-lexer ${Boost_LIBRARIES})
	target_link_libraries(test-asm-parser ${Boost_LIBRARIES})
	add_test(asm-lexer test-asm-lexer)
	add_test(asm-parser test-asm-parser)
endif()

add_library(catch-main main.cpp)

function(add_unit_test basepath)
	string(REPLACE "/" "-" test_name ${basepath})
	set(target_name test-${test_name})

	add_executable(${target_name} ${basepath}.cpp)
	set_target_properties(${target_name} PROPERTIES OUTPUT_NAME ${test_name})
	target_link_libraries(${target_name} catch-main llhd-common)

	add_test(${basepath} ${test_name} ${ARGN})
endfunction(add_unit_test)

add_unit_test(assembly/lexer)
add_unit_test(unicode/casefolding)
