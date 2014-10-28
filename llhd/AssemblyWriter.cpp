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
	out << "define " << in.getName() << " {\n";
	bool written = false;

	// Input and output.
	in.eachSignal([&](const AssemblySignal& s){
		if (s.getDirection() == AssemblySignal::kPortIn ||
			s.getDirection() == AssemblySignal::kPortOut) {
			out << '\t';
			write(s);
			written = true;
		}
	});
	if (written) out << '\n';
	written = false;

	// Signals.
	in.eachSignal([&](const AssemblySignal& s){
		if (s.getDirection() == AssemblySignal::kSignal) {
			out << '\t';
			write(s);
			written = true;
		}
	});
	if (written) out << '\n';
	written = false;

	// Registers.
	in.eachSignal([&](const AssemblySignal& s){
		if (s.getDirection() == AssemblySignal::kRegister) {
			out << '\t';
			write(s);
			written = true;
		}
	});
	if (written) out << '\n';
	written = false;

	// Assignments.
	in.eachInstruction([&](const AssemblyIns& ins){
		if (auto res = ins.getResult()) {
			out << '\t' << res->getName() << " = ";
		}
		write(ins);
		out << '\n';
		written = true;
	});

	out << "}\n";
	return *this;
}

void AssemblyWriter::write(const AssemblySignal& in) {
	switch (in.getDirection()) {
		case AssemblySignal::kPortIn:   out << "in ";   break;
		case AssemblySignal::kPortOut:  out << "out ";  break;
		case AssemblySignal::kSignal:   out << "wire "; break;
		case AssemblySignal::kRegister: out << "reg ";  break;
		default: return;
	}

	write(*in.getType());
	out << ' ' << in.getName() << '\n';
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
						out << uins.getArg()->getName();
					} else {
						out << "delay ";
						write(uins.getDelay());
						out << " " << uins.getArg()->getName();
					}
				} break;

				case AssemblyIns::kEdge: write("edge", uins); break;
				case AssemblyIns::kRisingEdge: write("rise", uins); break;
				case AssemblyIns::kFallingEdge: write("fall", uins); break;
				case AssemblyIns::kBoolNOT: write("not", uins); break;

				default:
					throw std::runtime_error("unknown unary opcode");
			}
		} break;

		// binary operations
		case AssemblyIns::kBinaryOps: {
			auto& bins = *(const AssemblyBinaryIns*)&in;
			switch (in.getOpcode()) {
				case AssemblyIns::kBoolAND:  write("and", bins); break;
				case AssemblyIns::kBoolOR:   write("or", bins); break;
				case AssemblyIns::kBoolNAND: write("nand", bins); break;
				case AssemblyIns::kBoolNOR:  write("nor", bins); break;
				case AssemblyIns::kBoolXOR:  write("xor", bins); break;
				case AssemblyIns::kStore:    write("st", bins); break;
				default:
					throw std::runtime_error("unknown binary opcode");
			}
		} break;

		// mux operations
		case AssemblyIns::kMuxOps: {
			switch (in.getOpcode()) {
				case AssemblyIns::kBimux: {
					auto& ins = *(const AssemblyBimuxIns*)&in;
					out << "bimux " << ins.getSelect()->getName() << ' ' <<
						ins.getCase0()->getName() << ' ' <<
						ins.getCase1()->getName();
				} break;
				default:
					throw std::runtime_error("unknown mux opcode");
			}
		} break;

		// catch unknown operation types
		default:
			throw std::runtime_error("unknown opcode type");
	}
}

void AssemblyWriter::write(const AssemblyDuration& in) {
	out << in << "ns";
}

void AssemblyWriter::write(const char* name, const AssemblyUnaryIns& in) {
	out << name << ' ' << in.getArg()->getName();
}

void AssemblyWriter::write(const char* name, const AssemblyBinaryIns& in) {
	out << name << ' ' << in.getArg0()->getName() << ' ' << in.getArg1()->getName();
}
