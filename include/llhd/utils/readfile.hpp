/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <string>
#include <vector>

namespace llhd {

bool readfile(const std::string &filename, std::vector<char> &dst);
bool readfile(const std::string &filename, std::string &dst);

} // namespace llhd
