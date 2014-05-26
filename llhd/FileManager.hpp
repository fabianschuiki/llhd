/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <string>

namespace llhd {

class FileEntry {
	std::string path;

public:
	const std::string& getPath() const { return path; }
	size_t getSize() const;
};

} // namespace llhd
