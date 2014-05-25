# Copyright (c) 2014 Fabian Schuiki
# Configures the various subcomponents of the LLHD codebase.

add_library(llhd-base
	SourceManager.cpp)

add_subdirectory(vhdl) # llhd-vhdl library
