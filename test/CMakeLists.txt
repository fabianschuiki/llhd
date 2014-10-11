# Copyright (c) 2014 Fabian Schuiki
# Configures the various unit tests of the LLHD codebase.

set(SRC ${CMAKE_SOURCE_DIR}/llhd)

add_executable(test-asm-lexer
	${SRC}/AssemblyLexer.cpp
	asm-lexer.cpp)
target_link_libraries(test-asm-lexer ${Boost_LIBRARIES})
add_test(asm-lexer test-asm-lexer)
