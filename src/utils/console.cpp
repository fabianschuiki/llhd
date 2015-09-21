/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/utils/console.hpp"
#include <sys/ioctl.h>
#include <unistd.h>

namespace llhd {

Console::Console(int fd) {
	struct winsize w;
	if (ioctl(fd, TIOCGWINSZ, &w) != 0)
		width = 0;
	else
		width = w.ws_col;

	has_colors = isatty(fd);
}

const Console kout(STDOUT_FILENO);
const Console kerr(STDERR_FILENO);

} // namespace llhd
