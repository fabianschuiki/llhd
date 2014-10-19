/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/AssemblyWriter.hpp"
#include <stdexcept>
using namespace llhd;

AssemblyWriter& AssemblyWriter::write(const Assembly& in) {
	bool first = true;
	for (auto& m : in.modules) {
		if (!first)
			out << "\n\n";
		write(*m.second);
	}
	return *this;
}

AssemblyWriter& AssemblyWriter::write(const AssemblyModule& in) {
	out << "define " << in.name << " {\n";
	bool written = false;

	// Input and output.
	for (auto& is : in.signals) {
		auto& s = *is.second;
		if (s.dir == AssemblySignal::kPortIn ||
			s.dir == AssemblySignal::kPortOut) {
			out << '\t';
			write(s);
			written = true;
		}
	}
	if (written) out << '\n';
	written = false;

	// Other signals.
	for (auto& is : in.signals) {
		auto& s = *is.second;
		if (s.dir == AssemblySignal::kSignal ||
			s.dir == AssemblySignal::kRegister) {
			out << '\t';
			write(s);
			written = true;
		}
	}
	if (written) out << '\n';
	written = false;

	// Assignments.
	for (auto& is : in.signals) {
		auto& s = *is.second;
		if (s.assignment) {
			out << '\t' << s.name << " = ";
			write(*s.assignment);
			out << '\n';
			written = true;
		}
	}

	out << "}\n";
	return *this;
}

void AssemblyWriter::write(const AssemblySignal& in) {
	switch (in.dir) {
		case AssemblySignal::kPortIn:   out << "in ";   break;
		case AssemblySignal::kPortOut:  out << "out ";  break;
		case AssemblySignal::kSignal:   out << "wire "; break;
		case AssemblySignal::kRegister: out << "reg ";  break;
		default: return;
	}

	write(*in.type);
	out << ' ' << in.name << '\n';
}

void AssemblyWriter::write(const AssemblyType& in) {
	if (dynamic_cast<const AssemblyTypeLogic*>(&in)) {
		out << "l1";
	}
	else if (auto type = dynamic_cast<const AssemblyTypeWord*>(&in)) {
		if (dynamic_cast<const AssemblyTypeLogic*>(type->type.get())) {
			out << 'l' << type->width;
		} else {
			write(*type->type);
			out << '[' << type->width << ']';
		}
	}
	else {
		throw std::runtime_error("unknown type");
	}
}

void AssemblyWriter::write(const AssemblyExpr& in) {
	if (auto e = dynamic_cast<const AssemblyExprIdentity*>(&in)) {
		out << e->op->name;
	}
	else if (auto e = dynamic_cast<const AssemblyExprDelayed*>(&in)) {
		out << "delay " << e->d << "ps " << e->op->name;
	}
	else if (auto e = dynamic_cast<const AssemblyExprBoolean*>(&in)) {
		switch (e->type) {
			case AssemblyExprBoolean::kAND:  out << "and ";  break;
			case AssemblyExprBoolean::kOR:   out << "or ";   break;
			case AssemblyExprBoolean::kNAND: out << "nand "; break;
			case AssemblyExprBoolean::kNOR:  out << "nor ";  break;
			case AssemblyExprBoolean::kXOR:  out << "xor ";  break;
		}
		out << e->op0->name << ' ' << e->op1->name;
	}
	else {
		throw std::runtime_error("unknown expression");
	}
}
