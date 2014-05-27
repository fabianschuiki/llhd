# Copyright (c) 2014 Fabian Schuiki
cmake_minimum_required(VERSION 2.8)
project(llhd)

# enable C++11 support and enable all warnings
if (UNIX)
	add_definitions(-Wall -std=c++11)
endif()

find_package(Boost COMPONENTS REQUIRED filesystem system)

enable_testing()
include_directories(.)
add_subdirectory(llhd)

# add a documentation target if Doxygen is available
find_package(Doxygen)
if (DOXYGEN_FOUND)
	configure_file(Doxyfile.in ${CMAKE_CURRENT_BINARY_DIR}/doc/Doxyfile @ONLY)
	add_custom_target(
		doc
		${DOXYGEN_EXECUTABLE} Doxyfile 1> doxygen.log 2> doxygen.err
		WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/doc
		COMMENT "Generating API documentation with Doxygen" VERBATIM)
endif()
