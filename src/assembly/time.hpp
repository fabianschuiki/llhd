/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <string>

namespace llhd {


/// \needsdoc
/// \ingroup assembly
class Time {
public:
	Time() = default;
	explicit Time(std::string literal) : m_literal(literal) {}

	friend std::string to_string(const Time &x) {
		return x.m_literal;
	}

private:
	std::string m_literal;
};


} // namespace llhd
