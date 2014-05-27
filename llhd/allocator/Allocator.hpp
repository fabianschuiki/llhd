/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"
#include <functional>

namespace llhd {

/// Base class and interface to all allocators in LLHD. Requires that
/// the class deriving from this (DerivedType) implements `allocate(size_t,
/// unsigned)` and `deallocate(size_t)`. The following convenience allocation
/// functions are provided:
///
/// - `one<T>(args...)` which allocates a single instance of T and passes its
///   constructor the arguments \a args.
/// - `oneRaw<T>()` which allocates a single instance of T without calling its
///   constructor. Useful for small classes that you will manually fill later.
/// - `many<T>(num)` which allocates \a num instances of T and calls the array
///   constructor.
/// - `many<T>(num, func)` which allocates \a num instances of T, calls the
///   array constructor, and calls the function \a func with a pointer and
///   index to each instance constructed.
/// - `manyRaw<T>(num)` which allocates \a num instances of T without calling
///   any constructor.
/// - `manyRaw<T>(num, func)` which allocates \a num instances of T without
///   calling any constructor, but calls the function \a func with a pointer
///   and index to each instance constructed.
template <typename DerivedType>
class Allocator {
	/// Calls the derived class' \c allocate function with \a size and \a
	/// alignment.
	inline void* derivedAllocate(size_t size, unsigned alignment = 0) {
		static_cast<DerivedType*>(this)->allocate(size, alignment);
	}

	/// Calls the derived class' \c deallocate function with \a ptr and \a
	/// size.
	inline void derivedDeallocate(void* ptr) {
		static_cast<DerivedType*>(this)->deallocate(ptr);
	}

public:
	/// Allocates an object \a T and calls its constructor with arguments \a
	/// args.
	template <typename T, typename... Args>
	T* one(Args&&... args) {
		T* t = (T*)derivedAllocate(sizeof(T), alignOf<T>::alignment);
		new (t) T(&args...);
		return t;
	}

	/// Allocates an object \a T without calling its constructor.
	template <typename T>
	T* oneRaw() {
		return (T*)derivedAllocate(sizeof(T), alignOf<T>::alignment);
	}

	/// Allocates \a num objects \a T and calls their array constructor.
	template <typename T>
	T* many(size_t num) {
		T* t = (T*)derivedAllocate(sizeof(T)*num, alignOf<T>::alignment);
		new (t) T[num];
		return t;
	}

	/// Allocates \a num objects \a T, calls their array constructor and calls
	/// the function \a func with a pointer and index to every object created.
	template <typename T>
	T* many(size_t num, std::function<void (T*, unsigned)> func) {
		T* t = (T*)derivedAllocate(sizeof(T)*num, alignOf<T>::alignment);
		new (t) T[num];
		T* p = t, e = t + num;
		for (unsigned i = 0; p != e; i++, p++)
			func(p, i);
		return t;
	}

	/// Allocates \a num objects \a T without calling their array constructor.
	template <typename T>
	T* manyRaw(size_t num) {
		return (T*)derivedAllocate(sizeof(T)*num, alignOf<T>::alignment);
	}

	/// Allocates \a num objects \a T without calling their array constructor,
	/// but calls the function \a func with a pointer and index to every object
	/// created.
	template <typename T>
	T* manyRaw(size_t num, std::function<void (T*, unsigned)> func) {
		T* t = (T*)derivedAllocate(sizeof(T)*num, alignOf<T>::alignment);
		T* p = t, e = t + num;
		for (unsigned i = 0; p != e; i++, p++)
			func(p, i);
		return t;
	}
};

} // namespace llhd
