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

	// Signals.
	for (auto& is : in.signals) {
		auto& s = *is.second;
		if (s.dir == AssemblySignal::kSignal) {
			out << '\t';
			write(s);
			written = true;
		}
	}
	if (written) out << '\n';
	written = false;

	// Registers.
	for (auto& is : in.signals) {
		auto& s = *is.second;
		if (s.dir == AssemblySignal::kRegister) {
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

void AssemblyWriter::write(const AssemblyIns& in) {
	switch (in.getOpcode() & AssemblyIns::kOpMask) {

		// unary operations
		case AssemblyIns::kUnaryOps: {
			auto& uins = *(const AssemblyUnaryIns*)&in;
			switch (in.getOpcode()) {

				// move
				case AssemblyIns::kMove: {
					if (uins.getDelay() == 0) {
						out << uins.getArg()->name;
					} else {
						out << "delay ";
						write(uins.getDelay());
						out << " " << uins.getArg()->name;
					}
				} break;

				default:
					throw std::runtime_error("unknown unary opcode");
			}
		} break;

		// binary operations
		case AssemblyIns::kBinaryOps: {
			auto& bins = *(const AssemblyBinaryIns*)&in;
			switch (in.getOpcode()) {
				case AssemblyIns::kBoolAND:  out << "and ";  break;
				case AssemblyIns::kBoolOR:   out << "or ";   break;
				case AssemblyIns::kBoolNAND: out << "nand "; break;
				case AssemblyIns::kBoolNOR:  out << "nor ";  break;
				case AssemblyIns::kBoolXOR:  out << "xor ";  break;
				default:
					throw std::runtime_error("unknown binary opcode");
			}
			out << bins.getArg0()->name << ' ' << bins.getArg1()->name;
		} break;

		// catch unknown operation types
		default:
			throw std::runtime_error("unknown opcode type");
	}
	// if (auto e = dynamic_cast<const AssemblyInsIdentity*>(&in)) {
	// 	out << e->op->name;
	// }
	// else if (auto e = dynamic_cast<const AssemblyInsDelayed*>(&in)) {
	// 	out << "delay " << e->d << "ps " << e->op->name;
	// }
	// else if (auto e = dynamic_cast<const AssemblyInsBoolean*>(&in)) {
	// 	switch (e->getOpcode()) {
	// 		case AssemblyIns::kBoolAND:  out << "and ";  break;
	// 		case AssemblyIns::kBoolOR:   out << "or ";   break;
	// 		case AssemblyIns::kBoolNAND: out << "nand "; break;
	// 		case AssemblyIns::kBoolNOR:  out << "nor ";  break;
	// 		case AssemblyIns::kBoolXOR:  out << "xor ";  break;
	// 		default:
	// 			throw std::runtime_error("unknown boolean opcode");
	// 	}
	// 	out << e->op0->name << ' ' << e->op1->name;
	// }
	// else {
	// 	throw std::runtime_error("unknown expression");
	// }
}

void AssemblyWriter::write(const AssemblyDuration& in) {
	out << in << "ns";
}
