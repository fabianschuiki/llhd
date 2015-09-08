# Copyright (c) 2014-2015 Fabian Schuiki
cmake_minimum_required(VERSION 2.8)
project(llhd)

if (CMAKE_BUILD_TYPE)
	message("## ${CMAKE_BUILD_TYPE} build")
endif()


set(LLHD_INCLUDE_DIR ${PROJECT_BINARY_DIR}/include)
set(LLHD_CONFIG_DIR ${PROJECT_BINARY_DIR}/config)
include_directories(${LLHD_INCLUDE_DIR} ${LLHD_CONFIG_DIR})


# Alias "src/" as "llhd/" in the include directories using a symlink.
add_custom_command(
	OUTPUT ${LLHD_INCLUDE_DIR}/llhd
	COMMAND ${CMAKE_COMMAND} -E make_directory ${LLHD_INCLUDE_DIR}
	COMMAND ${CMAKE_COMMAND} -E create_symlink ${PROJECT_SOURCE_DIR}/src ${LLHD_INCLUDE_DIR}/llhd
	COMMENT "Create symlink to source in include path")


# Generate the configuration header. Retrieve the current git commit and branch
# for nightly builds.
set(LLHD_VERSION_MAJOR 0)
set(LLHD_VERSION_MINOR 1)
set(LLHD_VERSION_PATCH 0)
set(LLHD_VERSION_SUFFIX "")
set(LLHD_VERSION "${LLHD_VERSION_MAJOR}.${LLHD_VERSION_MINOR}.${LLHD_VERSION_PATCH}${LLHD_VERSION_SUFFIX}")

execute_process(
	COMMAND git log -1 --format=%h
	WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
	OUTPUT_VARIABLE LLHD_GIT_COMMIT
	OUTPUT_STRIP_TRAILING_WHITESPACE)

execute_process(
	COMMAND git rev-parse --abbrev-ref HEAD
	WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
	OUTPUT_VARIABLE LLHD_GIT_BRANCH
	OUTPUT_STRIP_TRAILING_WHITESPACE)

option(LLHD_NIGHTLY "Make a cutting edge nightly build" OFF)
if (LLHD_NIGHTLY)
	execute_process(
		COMMAND date +%Y.%m.%d
		OUTPUT_VARIABLE DATE
		OUTPUT_STRIP_TRAILING_WHITESPACE)
	set(LLHD_VERSION "${LLHD_VERSION}-nightly.${DATE}+${GIT_COMMIT}")
endif()

message("## version ${LLHD_VERSION}")
configure_file(src/config.hpp.in ${LLHD_CONFIG_DIR}/llhd/config.hpp)


# Compiler Flags
set(CMAKE_CXX_FLAGS "${CMAKE_C_FLAGS} -std=c++11 -Wall")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -DLLHD_DEBUG")
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} -DLLHD_RELEASE")


# Add the build type "gcov" that produces a debug build with gcov output
# enabled. Useful for coverage analysis.
set(GCC_COVERAGE_COMPILER_FLAGS "-O0 -fprofile-arcs -ftest-coverage")
set(GCC_COVERAGE_LINKERR_FLAGS "-lgcov")

set(CMAKE_C_FLAGS_GCOV          "${CMAKE_C_FLAGS_DEBUG} ${GCC_COVERAGE_COMPILER_FLAGS}"        CACHE STRING "Flags used by the compiler during gcov-enabled debug builds")
set(CMAKE_CXX_FLAGS_GCOV        "${CMAKE_CXX_FLAGS_DEBUG} ${GCC_COVERAGE_COMPILER_FLAGS}"      CACHE STRING "Flags used by the compiler during gcov-enabled debug builds")
set(CMAKE_C_LINKER_FLAGS_GCOV   "${CMAKE_C_LINKER_FLAGS_DEBUG} ${GCC_COVERAGE_LINKER_FLAGS}"   CACHE STRING "Flags used by the linker during gcov-enabled debug builds")
set(CMAKE_CXX_LINKER_FLAGS_GCOV "${CMAKE_CXX_LINKER_FLAGS_DEBUG} ${GCC_COVERAGE_LINKER_FLAGS}" CACHE STRING "Flags used by the linker during gcov-enabled debug builds")

mark_as_advanced(
	CMAKE_C_FLAGS_GCOV
	CMAKE_CXX_FLAGS_GCOV
	CMAKE_C_LINKER_FLAGS_GCOV
	CMAKE_CXX_LINKER_FLAGS_GCOV)


# Compile the legacy code that is factored into the improved code base step by
# step.
option(LLHD_BUILD_LEGACY "Build the legacy code base" OFF)
if (LLHD_BUILD_LEGACY)
	find_package(Boost COMPONENTS REQUIRED system unit_test_framework)
	add_definitions(-DBOOST_TEST_DYN_LINK)
	include_directories(${Boost_INCLUDE_DIRS})

	include_directories(.)
	add_subdirectory(llhd)
endif()


# Build the libraries.
file(GLOB_RECURSE LLHD_COMMON_SOURCES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} src/assembly/*.cpp src/utils/*.cpp)
add_library(llhd-common ${LLHD_COMMON_SOURCES} ${LLHD_INCLUDE_DIR}/llhd)


# Build the tests if so requested.
option(LLHD_BUILD_TESTS "Build the unit tests" ON)
if (LLHD_BUILD_TESTS)
	enable_testing()
	add_subdirectory(test)
endif()


# Add a documentation target if requested and Doxygen is available.
option(LLHD_BUILD_DOC "Build the documentation" ON)
if (LLHD_BUILD_DOC)
	find_package(Doxygen)
	if (DOXYGEN_FOUND)
		configure_file(Doxyfile.in ${CMAKE_CURRENT_BINARY_DIR}/doc/Doxyfile @ONLY)
		add_custom_target(doc
			${DOXYGEN_EXECUTABLE} Doxyfile 1> doxygen.log 2> doxygen.err
			WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/doc
			COMMENT "Generating API documentation with Doxygen" VERBATIM
			SOURCES ${LLHD_INCLUDE_DIR}/llhd)
	endif()
endif()


# Installation
# install(TARGETS llhd-vhdl RUNTIME DESTINATION bin)
install(TARGETS llhd-common ARCHIVE DESTINATION lib)
install(DIRECTORY src/ DESTINATION include/llhd FILES_MATCHING PATTERN "*.hpp")
install(FILES ${LLHD_CONFIG_DIR}/llhd/config.hpp DESTINATION include/llhd)
