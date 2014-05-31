# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

# Generate the Unicode files.
add_executable(unicode-gen-casefolding unicode/gen-casefolding.cpp)
add_custom_command(
	OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding.cpp
	COMMAND unicode-gen-casefolding ${CMAKE_CURRENT_SOURCE_DIR}/unicode/CaseFolding.txt ${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding.cpp
	MAIN_DEPENDENCY ${CMAKE_CURRENT_SOURCE_DIR}/unicode/CaseFolding.txt
	DEPENDS unicode-gen-casefolding
	COMMENT "Generating unicode casefolding table")


add_library(llhd-base
	SourceManager.cpp
	unicode.cpp
	${CMAKE_CURRENT_BINARY_DIR}/unicode-casefolding.cpp)
target_link_libraries(llhd-base ${Boost_LIBRARIES})


add_subdirectory(vhdl) # llhd-vhdl library
