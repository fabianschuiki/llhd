# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

# Create the configuration header.
configure_file(config.hpp.in config.hpp)
include_directories(${PROJECT_BINARY_DIR})


# Generate the Unicode files.
add_executable(unicode-tblgen-casefolding unicode/tblgen-casefolding.cpp)
add_custom_command(
	OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding-table.cpp
	COMMAND unicode-tblgen-casefolding
		${CMAKE_CURRENT_SOURCE_DIR}/unicode/CaseFolding.txt
		${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding-table.cpp
		> ${CMAKE_CURRENT_BINARY_DIR}/unicode-tblgen-casefolding.log
	MAIN_DEPENDENCY ${CMAKE_CURRENT_SOURCE_DIR}/unicode/CaseFolding.txt
	DEPENDS unicode-tblgen-casefolding
	COMMENT "Generating unicode casefolding table")


add_library(llhd-base
	console.cpp
	SourceLocation.cpp
	SourceManager.cpp
	SourceManagerEntry.cpp
	diagnostic/DiagnosticFormatterConsole.cpp
	unicode/casefolding.cpp
	${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding-table.cpp)
target_link_libraries(llhd-base ${Boost_LIBRARIES})

add_subdirectory(unicode)
add_subdirectory(vhdl) # llhd-vhdl library

# sample simulator
add_executable(llhd-sim
	AssemblyLexer.cpp
	AssemblyParser.cpp
	console.cpp
	diagnostic/DiagnosticFormatterConsole.cpp
	sim.cpp
	SourceLocation.cpp
	SourceManager.cpp
	SourceManagerEntry.cpp
)

# sample simulator
add_executable(llhd-sim2
	AssemblyWriter.cpp
	sim/Simulation.cpp
	sim2.cpp
)

# add tests
add_executable(test-diagnostics test/diagnostics.cpp)
add_executable(test-sourcemanager test/sourcemanager.cpp)
add_executable(test-sourcerangeset test/sourcerangeset.cpp)
target_link_libraries(test-diagnostics llhd-base)
target_link_libraries(test-sourcemanager llhd-base)
target_link_libraries(test-sourcerangeset llhd-base)
add_test(diagnostics test-diagnostics)
add_test(sourcemanager test-sourcemanager)
add_test(sourcerangeset test-sourcerangeset)
