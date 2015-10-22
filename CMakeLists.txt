# Copyright (c) 2014-2015 Fabian Schuiki
cmake_minimum_required(VERSION 2.8)
project(llhd)

if (CMAKE_BUILD_TYPE)
	message("## llhd ${CMAKE_BUILD_TYPE} build")
endif()


set(LLHD_INCLUDE_DIR ${PROJECT_BINARY_DIR}/include)
set(LLHD_PUBLIC_INCLUDE_DIR ${PROJECT_SOURCE_DIR}/include)
set(LLHD_CONFIG_DIR ${PROJECT_BINARY_DIR}/config)
include_directories(${LLHD_INCLUDE_DIR} ${LLHD_PUBLIC_INCLUDE_DIR} ${LLHD_CONFIG_DIR} ${PROJECT_SOURCE_DIR})


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

message("## llhd version ${LLHD_VERSION}")
configure_file(src/config.hpp.in ${LLHD_CONFIG_DIR}/llhd/config.hpp)


# Compiler Flags
set(CMAKE_CXX_FLAGS "-std=c++11 -Wall -Wextra -Wno-parentheses" CACHE STRING "C++ compiler flags" FORCE)
set(CMAKE_CXX_FLAGS_COVERAGE "-O0 -g --coverage")
mark_as_advanced(CMAKE_CXX_FLAGS_COVERAGE)


# Specify additional targets in case coverage analysis is requested.
# - coverage-init establishes a baseline coverage
# - coverage captures coverage information after tests have run
# - coverage-report generates an HTML report
if (CMAKE_BUILD_TYPE STREQUAL "coverage")
	find_program(GCOV_PATH gcov REQUIRED)
	find_program(LCOV_PATH lcov REQUIRED)
	find_program(GENHTML_PATH genhtml)

	mark_as_advanced(GCOV_PATH LCOV_PATH GENHTML_PATH)

	if (NOT DEFINED COVERAGE_INFO_PATH)
		set(COVERAGE_INFO_PATH ${CMAKE_BINARY_DIR}/coverage.info)
	endif()

	if (NOT DEFINED COVERAGE_HTML_PATH)
		set(COVERAGE_HTML_PATH ${CMAKE_BINARY_DIR}/coverage)
	endif()

	if (NOT DEFINED LCOV_LOG)
		set(LCOV_LOG ${CMAKE_BINARY_DIR}/coverage.log)
	endif()

	if (NOT DEFINED LCOV_COMMAND)
		set(LCOV_COMMAND ${LCOV_PATH} --gcov-tool ${GCOV_PATH} --base-directory ${CMAKE_SOURCE_DIR} --directory ${CMAKE_BINARY_DIR} --no-external --output-file ${COVERAGE_INFO_PATH}.raw)
	endif()

	add_custom_target(coverage-init
		COMMAND ${LCOV_COMMAND} --zerocounters >${LCOV_LOG}
	)
	add_custom_target(coverage
		COMMAND ${LCOV_COMMAND} --capture >>${LCOV_LOG}
		COMMAND sed -e 's:${LLHD_INCLUDE_DIR}/llhd:${CMAKE_SOURCE_DIR}/src:'
		            -e 's:${LLHD_CONFIG_DIR}/llhd:${CMAKE_SOURCE_DIR}/src:'
		            <${COVERAGE_INFO_PATH}.raw >${COVERAGE_INFO_PATH}.replaced
		COMMAND ${LCOV_PATH} --remove ${COVERAGE_INFO_PATH}.replaced 'test/*' 'deps/*' --output-file ${COVERAGE_INFO_PATH} >>${LCOV_LOG}
	)

	if (GENHTML_PATH)
		add_custom_target(coverage-report COMMAND ${GENHTML_PATH} -o ${COVERAGE_HTML_PATH} ${COVERAGE_INFO_PATH} >>${LCOV_LOG})
	endif()
endif()


# Compile the legacy code that is factored into the improved code base step by
# step.
option(LLHD_BUILD_LEGACY "Build the legacy code base" OFF)
if (LLHD_BUILD_LEGACY)
	find_package(Boost COMPONENTS REQUIRED system unit_test_framework)
	add_definitions(-DBOOST_TEST_DYN_LINK)
	include_directories(${Boost_INCLUDE_DIRS})

	add_subdirectory(llhd)
endif()


# Generate the unicode casefolding table.
set(LLHD_UNICODE_CASEFOLDING_INPUT  ${CMAKE_CURRENT_SOURCE_DIR}/src/unicode/CaseFolding.txt)
set(LLHD_UNICODE_CASEFOLDING_OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/casefolding-table.cpp)
add_executable(casefolding-tablegen src/unicode/casefolding-tablegen.cpp)
add_custom_command(
	OUTPUT ${LLHD_UNICODE_CASEFOLDING_OUTPUT}
	COMMAND casefolding-tablegen ${LLHD_UNICODE_CASEFOLDING_INPUT} ${LLHD_UNICODE_CASEFOLDING_OUTPUT} > ${CMAKE_CURRENT_BINARY_DIR}/casefolding-tablegen.log
	MAIN_DEPENDENCY ${LLHD_UNICODE_CASEFOLDING_INPUT}
	DEPENDS casefolding-tablegen
	COMMENT "Generating unicode casefolding table")


# Build the libraries.
include_directories(${PROJECT_SOURCE_DIR}/deps/catch)
file(GLOB_RECURSE LLHD_COMMON_SOURCES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} src/assembly/*.cpp src/diagnostic/*.cpp src/utils/*.cpp)
list(APPEND LLHD_COMMON_SOURCES
	src/unicode/casefolding.cpp
	${LLHD_UNICODE_CASEFOLDING_OUTPUT})
add_library(llhd-common ${LLHD_COMMON_SOURCES} ${LLHD_INCLUDE_DIR}/llhd)


# Build the programs.
file(GLOB_RECURSE LLHD_TOOL_SOURCES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} src/tool/*.cpp)
add_executable(llhd-tool ${LLHD_TOOL_SOURCES} ${LLHD_INCLUDE_DIR}/llhd)
target_link_libraries(llhd-tool llhd-common)


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
install(TARGETS llhd-tool RUNTIME DESTINATION bin)
install(TARGETS llhd-common ARCHIVE DESTINATION lib)
install(DIRECTORY src/ DESTINATION include/llhd FILES_MATCHING PATTERN "*.hpp")
install(FILES ${LLHD_CONFIG_DIR}/llhd/config.hpp DESTINATION include/llhd)
