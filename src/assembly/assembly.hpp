/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/utils/memory.hpp"
#include "llhd/assembly/function.hpp"
#include "llhd/assembly/module.hpp"
#include "llhd/assembly/process.hpp"
#include <string>
#include <vector>

namespace llhd {


/// \needsdoc
/// \ingroup assembly
class Statement {
public:
	Statement() = default;

	template <typename T>
	Statement(T x) : m_self(std::make_shared<Model<T>>(std::move(x))) {}

	friend std::string to_string(const Statement &x) { return x.m_self->to_string_(); }
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
class Assembly {
public:
	std::vector<Statement> statements;

	friend std::string to_string(const Assembly &x) {
		auto i = x.statements.begin();
		if (i == x.statements.end())
			return std::string();
		std::string r = to_string(*i);
		for (; i != x.statements.end(); ++i)
			r += "\n\n" + to_string(*i);
		return r;
	}
};

} // namespace llhd
