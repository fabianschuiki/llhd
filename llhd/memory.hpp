/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <memory>
#include <utility>

/// \file
/// Includes and extends the memory management facilities contained in the
/// standard library's <memory> header.

namespace llhd {

/// Returns a std::unique_ptr to a newly constructed instance of \a T, passing
/// \a args to the class' constructor.
template<typename T, typename... Args>
std::unique_ptr<T>
make_unique (Args&&... args) {
	return std::unique_ptr<T>(new T(std::forward<Args>(args)...));
}

} // namespace llhd
