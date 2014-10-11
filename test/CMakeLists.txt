# Copyright (c) 2014 Fabian Schuiki
# Configures the various unit tests of the LLHD codebase.

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
