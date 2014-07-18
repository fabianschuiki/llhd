/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"
#include <memory>

namespace llhd {
namespace vhdl {
namespace ast {

#define one(type, name) std::unique_ptr<type> name;
#define many(type, name) std::vector<std::unique_ptr<type>> name;

struct Node {
	SourceRange range;
};

struct Identifier : public Node {
	std::string value;
};

struct ContextItem;
struct DesignFile;
struct DesignUnit;
struct LibraryClause;
struct LibraryUnit;
struct Prefix;
struct SelectedNamed;
struct Suffix;

struct DesignFile : public Node {
	many (DesignUnit, units)
};

struct DesignUnit : public Node {
	many (ContextItem, contextItems)
	one (LibraryUnit, libraryUnit)
};

struct ContextItem : public Node {};

struct LibraryClause : public ContextItem {
	std::vector<Identifier> names;
};

struct UseClause : public ContextItem {
	many (SelectedName, names)
};

struct SelectedName : public Node {
	one (Prefix, prefix)
	one (Suffix, suffix)
};

#undef one
#undef many

} // namespace ast
} // namespace vhdl
} // namespace llhd
