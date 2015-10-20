/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/utils/memory.hpp"
#include <string>

namespace llhd {


/// \needsdoc
/// \ingroup assembly
class Value {
public:
	Value() = default;

	template <typename T>
	Value(T x) : m_self(std::make_shared<Model<T>>(std::move(x))) {}

	friend std::string to_string(const Value &x) { return x.m_self->to_string_(); }
	explicit operator bool() const { return bool(m_self); }

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


/// \needsdoc
/// \ingroup assembly
class UnresolvedValue {
public:
	UnresolvedValue(std::string name) : m_name(name) {}

	friend std::string to_string(const UnresolvedValue &x) {
		return x.m_name;
	}

private:
	std::string m_name;
};


/// \needsdoc
/// \ingroup assembly
class NumberValue {
public:
	NumberValue(std::string literal) : m_literal(literal) {}

	friend std::string to_string(const NumberValue &x) {
		return x.m_literal;
	}

private:
	std::string m_literal;
};


} // namespace llhd
