/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/console.hpp"
#include <sys/ioctl.h>
#include <unistd.h>

namespace llhd {

/// Returns the width of the STDOUT terminal, or 0.
unsigned getTerminalWidth() {
    struct winsize w;
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) != 0)
        return 0;
    return w.ws_col;
}

} // namespace llhd
