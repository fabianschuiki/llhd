# Copyright (c) 2014-2015 Fabian Schuiki
cmake_minimum_required(VERSION 2.6)
project(llhd)

if (CMAKE_BUILD_TYPE)
	message("## ${CMAKE_BUILD_TYPE} build")
endif()

# Project version.
set(VERSION_MAJOR 0)
set(VERSION_MINOR 1)
set(VERSION_PATCH 0)


# Enable C++11 support and enable all warnings.
if (UNIX)
	add_definitions(-Wall)
	set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
endif()


# Add the build type "gcov" that produces a debug build with gcov output
# enabled. Useful for coverage analysis.
set(GCC_COVERAGE_COMPILER_FLAGS "-O0 -fprofile-arcs -ftest-coverage")
set(GCC_COVERAGE_LINKERR_FLAGS "-lgcov")

set(CMAKE_C_FLAGS_GCOV
	"${CMAKE_C_FLAGS_DEBUG} ${GCC_COVERAGE_COMPILER_FLAGS}"
	CACHE STRING "Flags used by the compiler during gcov-enabled debug builds")
set(CMAKE_CXX_FLAGS_GCOV
	"${CMAKE_CXX_FLAGS_DEBUG} ${GCC_COVERAGE_COMPILER_FLAGS}"
	CACHE STRING "Flags used by the compiler during gcov-enabled debug builds")
set(CMAKE_C_LINKER_FLAGS_GCOV
	"${CMAKE_C_LINKER_FLAGS_DEBUG} ${GCC_COVERAGE_LINKER_FLAGS}"
	CACHE STRING "Flags used by the linker during gcov-enabled debug builds")
set(CMAKE_CXX_LINKER_FLAGS_GCOV
	"${CMAKE_CXX_LINKER_FLAGS_DEBUG} ${GCC_COVERAGE_LINKER_FLAGS}"
	CACHE STRING "Flags used by the linker during gcov-enabled debug builds")

mark_as_advanced(
	CMAKE_C_FLAGS_GCOV
	CMAKE_CXX_FLAGS_GCOV
	CMAKE_C_LINKER_FLAGS_GCOV
	CMAKE_CXX_LINKER_FLAGS_GCOV)

# Add a target to clear all gcov output files.
add_custom_target(clean-gcov
	find . -name '*.gcov' -or -name '*.gcda' | xargs rm
	WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})


find_package(Boost COMPONENTS REQUIRED system unit_test_framework)
add_definitions(-DBOOST_TEST_DYN_LINK)
include_directories(${Boost_INCLUDE_DIRS})

enable_testing()
include_directories(.)
add_subdirectory(llhd)
add_subdirectory(test)
add_subdirectory(exp)

# Add a documentation target if Doxygen is available.
find_package(Doxygen)
if (DOXYGEN_FOUND)
	configure_file(Doxyfile.in ${CMAKE_CURRENT_BINARY_DIR}/doc/Doxyfile @ONLY)
	add_custom_target(
		doc
		${DOXYGEN_EXECUTABLE} Doxyfile 1> doxygen.log 2> doxygen.err
		WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/doc
		COMMENT "Generating API documentation with Doxygen" VERBATIM)
endif()
