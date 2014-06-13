# Copyright (c) 2014 Fabian Schuiki
# Configure the unicode unit tests.
add_executable(test-unicode-casefolding test/casefolding.cpp)
target_link_libraries(test-unicode-casefolding llhd-base ${Boost_UNIT_TEST_FRAMEWORK_LIBRARY})
add_test(unicode-casefolding test-unicode-casefolding)
