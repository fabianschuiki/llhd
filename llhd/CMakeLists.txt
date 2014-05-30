# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

add_executable(unicode-gen-casefolding unicode/gen-casefolding.cpp)

add_library(llhd-base
	SourceManager.cpp
	unicode.cpp
	${CMAKE_CURRENT_BINARY_DIR}/unicode/casefolding.cpp)
target_link_libraries(llhd-base ${Boost_LIBRARIES})

add_subdirectory(vhdl) # llhd-vhdl library
