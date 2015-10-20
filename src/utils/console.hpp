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

	#define COLORS(N,B) \
	static const unsigned N##_def          = B+9; \
	static const unsigned N##_black        = B+0; \
	static const unsigned N##_red          = B+1; \
	static const unsigned N##_green        = B+2; \
	static const unsigned N##_yellow       = B+3; \
	static const unsigned N##_blue         = B+4; \
	static const unsigned N##_magenta      = B+5; \
	static const unsigned N##_cyan         = B+6; \
	static const unsigned N##_lightgray    = B+7; \
	static const unsigned N##_darkgray     = B+60; \
	static const unsigned N##_lightred     = B+61; \
	static const unsigned N##_lightgreen   = B+62; \
	static const unsigned N##_lightyellow  = B+63; \
	static const unsigned N##_lightblue    = B+64; \
	static const unsigned N##_lightmagenta = B+65; \
	static const unsigned N##_lightcyan    = B+66; \
	static const unsigned N##_white        = B+67;

	/// Foreground color modifiers.
	COLORS(fg,30);
	/// Background color modifiers.
	COLORS(bg,40);
	#undef COLORS

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
