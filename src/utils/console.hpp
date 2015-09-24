/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <string>

namespace llhd {

class Console {
public:
	bool has_colors;
	unsigned width;

	Console(int fd);

	template<typename... Args>
	std::string format(unsigned f0, Args... args) const {
		std::string out = "\033[" + std::to_string(f0);
		inner_format(out, args...);
		return out;
	}

	// Formatting
	static const unsigned bold        = 1;
	static const unsigned dim         = 2;
	static const unsigned underline   = 4;
	static const unsigned blink       = 5;
	static const unsigned invert      = 7;
	static const unsigned hide        = 8;

	static const unsigned reset       = 0;
	static const unsigned unbold      = 21;
	static const unsigned undim       = 22;
	static const unsigned ununderline = 24;
	static const unsigned unblink     = 25;
	static const unsigned uninvert    = 27;
	static const unsigned unhide      = 28;

	#define COLORS_STRUCT(B) \
	struct { \
		const unsigned def          = B+9; \
		const unsigned black        = B+0; \
		const unsigned red          = B+1; \
		const unsigned green        = B+2; \
		const unsigned yellow       = B+3; \
		const unsigned blue         = B+4; \
		const unsigned magenta      = B+5; \
		const unsigned cyan         = B+6; \
		const unsigned lightgray    = B+7; \
		const unsigned darkgray     = B+60; \
		const unsigned lightred     = B+61; \
		const unsigned lightgreen   = B+62; \
		const unsigned lightyellow  = B+63; \
		const unsigned lightblue    = B+64; \
		const unsigned lightmagenta = B+65; \
		const unsigned lightcyan    = B+66; \
		const unsigned white        = B+67; \
	}

	/// Foreground color modifiers.
	static const COLORS_STRUCT(30) fg;
	/// Background color modifiers.
	static const COLORS_STRUCT(40) bg;

	#undef COLORS_STRUCT

private:
	template<typename... Args>
	void inner_format(std::string& out, unsigned f0, Args... args) const {
		out += ';' + std::to_string(f0);
		inner_format(out, args...);
	}

	void inner_format(std::string& out) const {
		out += "m";
	}
};

extern const Console kout;
extern const Console kerr;

} // namespace llhd
