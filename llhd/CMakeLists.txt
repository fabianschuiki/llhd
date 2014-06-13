# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

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
	SourceManager.cpp
	unicode/casefolding.cpp
	${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding-table.cpp)
target_link_libraries(llhd-base ${Boost_LIBRARIES})

add_subdirectory(unicode)
add_subdirectory(vhdl) # llhd-vhdl library
