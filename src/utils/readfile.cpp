/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/utils/readfile.hpp"
#include <iterator>
#include <streambuf>
#include <fstream>

namespace llhd {

bool readfile(const std::string &filename, std::vector<char> &dst) {
	std::ifstream f(filename.c_str());
	if (!f.good())
		return false;
	f.seekg(0, std::ios::end);
	size_t l = f.tellg();
	f.seekg(0, std::ios::beg);
	dst.reserve(dst.size() + l);
	dst.insert(dst.end(), std::istreambuf_iterator<char>(f), std::istreambuf_iterator<char>());
	return true;
}

bool readfile(const std::string &filename, std::string &dst) {
	std::ifstream f(filename.c_str());
	if (!f.good())
		return false;
	f.seekg(0, std::ios::end);
	size_t l = f.tellg();
	f.seekg(0, std::ios::beg);
	dst.reserve(dst.size() + l);
	dst.append(std::istreambuf_iterator<char>(f), std::istreambuf_iterator<char>());
	return true;
}

} // namespace llhd
