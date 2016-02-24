// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdio.h>
#include <stdint.h>

#if defined(_WIN32) && defined(LLHD_BUILD_SHARED)
	#define LLHD_API __declspec(dllexport)
#elif defined(_WIN32) && defined(LLHD_DLL)
	#define LLHD_API __declspec(dllimport)
#elif defined(__GNUC__) && defined(LLHD_BUILD_SHARED)
	#define LLHD_API __attribute__((visibility("default")))
#else
	#define LLHD_API
#endif
