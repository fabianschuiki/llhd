/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"
#include "llhd/utils/memory.hpp"
#include <string>

namespace llhd {


/// \todo This is just a dummy. Remove this later on.
class UnknownType {
	std::string m_name;
public:
	explicit UnknownType(std::string name) : m_name(name) {}

	friend std::string to_string(const UnknownType &x) {
		return x.m_name;
	}
};


/// \needsdoc
/// \ingroup assembly
class Type {
public:
	Type() = default;

	template <typename T>
	Type(T x) : m_self(std::make_shared<Model<T>>(std::move(x))) {}

	friend std::string to_string(const Type &x) { return x.m_self->to_string_(); }

private:
	struct Concept {
		virtual ~Concept() = default;
		virtual std::string to_string_() const = 0;
	};

	template <typename T>
	struct Model : Concept {
		Model(T x) : x(std::move(x)) {}
		virtual std::string to_string_() const override { return to_string(x); }
		T x;
	};

	std::shared_ptr<const Concept> m_self;
};


} // namespace llhd
