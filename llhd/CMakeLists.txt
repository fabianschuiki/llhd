# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

add_library(llhd-base
	SourceManager.cpp)
target_link_libraries(llhd-base ${Boost_LIBRARIES})

add_subdirectory(vhdl) # llhd-vhdl library
