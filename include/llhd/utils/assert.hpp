/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <cassert>

#define llhd_static_assert(expr) static_assert(expr, #expr)
#define llhd_static_assert_msg(expr, msg) static_assert(expr, msg)
#define llhd_assert(expr) assert(expr)
#define llhd_assert_msg(expr, msg) assert(expr && msg)
#define llhd_abort() llhd_assert(false)
#define llhd_abort_msg(msg) llhd_assert_msg(false, msg)
#define llhd_unimplemented() llhd_abort_msg("not implemented")
