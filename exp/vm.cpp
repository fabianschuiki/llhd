/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/sim/SimulationTime.hpp"
#include <cassert>
#include <iostream>
#include <map>
#include <memory>
#include <set>
#include <stack>
#include <string>
#include <vector>

class BasicInstruction;
class Process;


// Supported values:
// - signed int of arbitrary width
// - unsigned int of arbitrary width
// - fixed-width optimizations of the above
// - nine-value logic word of arbitrary width
// - arrays of values

class Value {
public:
	virtual unsigned get_width() const = 0;
};


class UnsignedValue : public Value {
	unsigned width;
	union {
		uint64_t *chunks;
		uint64_t value;
	};

	inline unsigned num_chunks() const {
		return width / 64;
	}

	inline bool is_compact() const {
		return width <= 64;
	}

public:
	UnsignedValue(unsigned width) {
		this->width = width;
		if (!is_compact()) {
			chunks = new uint64_t[num_chunks()];
		}
	}

	UnsignedValue(const UnsignedValue &other) {
		UnsignedValue(other.width);
		if (is_compact()) {
			value = other.value;
		} else {
			std::copy(other.chunks, other.chunks+num_chunks(), chunks);
		}
	}

	UnsignedValue(UnsignedValue &&other) {
		width = other.width;
		chunks = other.chunks; // doubles as value copy
		other.chunks = nullptr;
	}

	~UnsignedValue() {
		if (is_compact() && chunks) {
			delete[] chunks;
			chunks = nullptr;
		}
	}

	virtual unsigned get_width() const { return width; }
};


class LogicValue : public Value {
	unsigned width;
	union {
		uint8_t *chunks;
		uint8_t value[8];
	};

	inline bool is_compact() const {
		return width <= 8;
	}

public:
	LogicValue(unsigned width) {
		this->width = width;
		if (!is_compact()) {
			chunks = new uint8_t[width];
		}
	}

	LogicValue(const LogicValue &other) {
		LogicValue(other.width);
		if (is_compact()) {
			std::copy(other.value, other.value+width, value);
		} else {
			std::copy(other.chunks, other.chunks+width, chunks);
		}
	}

	LogicValue(LogicValue &&other) {
		width = other.width;
		chunks = other.chunks; // doubles as value copy
		other.chunks = nullptr;
	}

	~LogicValue() {
		if (is_compact() && chunks) {
			delete[] chunks;
			chunks = nullptr;
		}
	}

	virtual unsigned get_width() const { return width; }
};


class Bitmask {
	unsigned width;
	union {
		uint64_t *chunks;
		uint64_t value;
	};

	inline bool is_compact() const {
		return width <= 64;
	}

	inline unsigned num_chunks() const {
		return width / 64;
	}

public:
	Bitmask operator~() const {
		return *this;
	}

	Bitmask& operator&=(const Bitmask &other) {
		return *this;
	}

	Bitmask& operator|=(const Bitmask &other) {
		return *this;
	}

	bool is_all_zero() const {
		return false;
	}
};


class Event {
public:
	unsigned target;
	llhd::SimulationTime time;
	std::unique_ptr<Value> value;
	Bitmask mask;
};


class EventQueue {
	std::vector<Event> events;
	std::vector<Event> added_events;

	static bool compare_events(const Event &a, const Event &b) {
		if (b.mask.is_all_zero() && !a.mask.is_all_zero())
			return true;
		return a.time < b.time;
	}

public:
	void add(Event &&event) {
		/// \todo Iterate over added_events and mask earlier events where
		/// appropriate.
		for (auto &ae : added_events) {
			if (ae.target == event.target && ae.time >= event.time)
				ae.mask &= ~event.mask;
		}
		added_events.push_back(std::move(event));
	}

	void commit() {
		std::sort(added_events.begin(), added_events.end(), compare_events);

		std::map<unsigned,Bitmask> seen;
		auto ai = added_events.begin();
		auto ae = added_events.end();

		for (auto &e : events) {
			while (ai != ae && ai->time <= e.time) {
				auto se = seen.find(ai->target);
				if (se == seen.end())
					seen.insert(std::make_pair(ai->target, ai->mask));
				else
					se->second |= ai->mask;
				++ai;
			}

			auto se = seen.find(e.target);
			if (se != seen.end()) {
				e.mask &= ~se->second;
			}
		}

		events.insert(events.end(),
			std::make_move_iterator(added_events.begin()),
			std::make_move_iterator(added_events.end())
		);
		added_events.clear();

		std::sort(events.begin(), events.end(), compare_events);
		while (!events.empty() && events.back().mask.is_all_zero())
			events.pop_back();
	}
};


// TODO: Rewrite the virtual machine to cover the following instructions:
// Load and store from memory into registers. Input address lookup. Arithmetic,
// logic, and branch instructions on registers. Drive instruction for outputs.
// Wait instruction that waits for changing inputs.

enum InsOp {
	INS_MASK_GRP = 0xFF00,
	INS_MASK_OP  = 0x00FF,

	INS_GRP_LD    = 0x100,
	INS_GRP_CMP   = 0x200,
	INS_GRP_BR    = 0x300,
	INS_GRP_ARI   = 0x400,
	INS_GRP_LOG   = 0x500,
	INS_GRP_WAIT  = 0x600,
	INS_GRP_DBG   = 0xF00,

	INS_OP_LD     = 0x100,
	INS_OP_IN     = 0x101,
	INS_OP_DRV    = 0x102,

	INS_OP_CMPEQ  = 0x200,
	INS_OP_CMPNEQ = 0x201,
	INS_OP_CMPLT  = 0x202,
	INS_OP_CMPGT  = 0x203,
	INS_OP_CMPLEQ = 0x204,
	INS_OP_CMPGEQ = 0x205,

	INS_OP_BR     = 0x300,
	INS_OP_BRC    = 0x301,
	INS_OP_BRCE   = 0x302,

	INS_OP_ADD    = 0x400,
	INS_OP_SUB    = 0x401,
	INS_OP_MUL    = 0x402,
	INS_OP_DIV    = 0x403,

	INS_OP_NEG    = 0x500,
	INS_OP_AND    = 0x501,
	INS_OP_OR     = 0x502,
	INS_OP_XOR    = 0x503,

	INS_OP_WAITA  = 0x600,
	INS_OP_WAITR  = 0x601,
	INS_OP_WAITW  = 0x602,

	INS_OP_DBG    = 0xF00,
};

enum InsParamType {
	INS_TYPE_NONE = 0x0,
	INS_TYPE_U8   = 0x1, // 8bit unsigned
	INS_TYPE_S8   = 0x2, // 8bit signed
	INS_TYPE_U16  = 0x3, // 16bit unsigned
	INS_TYPE_S16  = 0x4, // 16bit signed
	INS_TYPE_U32  = 0x5, // 32bit unsigned
	INS_TYPE_S32  = 0x6, // 32bit signed
	INS_TYPE_U64  = 0x7, // 64bit unsigned
	INS_TYPE_S64  = 0x8, // 64bit signed
	INS_TYPE_L    = 0x9, // logic vector
	INS_TYPE_T    = 0xA, // simulation time
	INS_TYPE_F32  = 0xB, // float 32bit
	INS_TYPE_F64  = 0xC, // float 64bit
};

// static std::string const op0_names[] = {
// 	"LD", "CMP", "BR", "AR", "DBG"
// };

// static std::string const op1_cmp_names[] = {
// 	"EQ", "NEQ", "LT", "GT", "LEQ", "GEQ"
// };

// static std::string const op1_br_names[] = {
// 	"", "C", "CE"
// };

// static std::string const op1_ar_names[] = {
// 	"ADD", "SUB", "MUL", "DIV", "MOD"
// };

// static const std::string* const op1_names[] = {
// 	nullptr, op1_cmp_names, nullptr, nullptr, nullptr
// };


// TODO: Do not use registers but rather address into the memory directly. Have
// the instructions carry type information, e.g. an AND for 8 bit, 16 bit,
// 32 bit signed and unsigned integers, as well as for logic vectors and
// booleans.

// TODO: Add inputs and outputs such that processes may react to changing
// signals and in turn induce changes to signals. Add a wait instruction that
// suspends a process until the input signals change or a desired point in time
// has been reached. Add a drive instruction that schedules a signal change in
// the event queue for the current time step, the next delta time step, or any
// other time in the future.

enum InsParamMode {
	INS_MODE_NONE = 0x0,
	INS_MODE_REG  = 0x1,
	INS_MODE_IMM  = 0x2,
	INS_MODE_MEM  = 0x3,
};

class Instruction {
public:
	union {
		uint32_t opcode = 0;
		struct {
			unsigned op:16;
			unsigned type:8;
			unsigned md:2;
			unsigned ma:2;
			unsigned mb:2;
		};
	};
	uint64_t pd = 0;
	uint64_t pa = 0;
	uint64_t pb = 0;

	friend std::ostream& operator<< (std::ostream& o, Instruction i) {
		auto f = o.flags();
		// o.width(8);
		// o << std::left;
		switch (i.op) {
			case INS_OP_LD: o << "LD"; break;
			case INS_OP_IN: o << "IN"; break;
			case INS_OP_DRV: o << "DRV"; break;
			case INS_OP_CMPEQ: o << "CMPEQ"; break;
			case INS_OP_CMPNEQ: o << "CMPNEQ"; break;
			case INS_OP_CMPLT: o << "CMPLT"; break;
			case INS_OP_CMPGT: o << "CMPGT"; break;
			case INS_OP_CMPLEQ: o << "CMPLEQ"; break;
			case INS_OP_CMPGEQ: o << "CMPGEQ"; break;
			case INS_OP_BR: o << "BR"; break;
			case INS_OP_BRC: o << "BRC"; break;
			case INS_OP_BRCE: o << "BRCE"; break;
			case INS_OP_ADD: o << "ADD"; break;
			case INS_OP_SUB: o << "SUB"; break;
			case INS_OP_MUL: o << "MUL"; break;
			case INS_OP_DIV: o << "DIV"; break;
			case INS_OP_NEG: o << "NEG"; break;
			case INS_OP_AND: o << "AND"; break;
			case INS_OP_OR: o << "OR"; break;
			case INS_OP_XOR: o << "XOR"; break;
			case INS_OP_WAITA: o << "WAITA"; break;
			case INS_OP_WAITR: o << "WAITR"; break;
			case INS_OP_WAITW: o << "WAITW"; break;
			case INS_OP_DBG: o << "DBG"; break;
			default:
				o << std::hex << i.op;
				break;
		}
		o.flags(f);

		switch (i.type) {
			case INS_TYPE_U8: o << ".U8"; break;
			case INS_TYPE_S8: o << ".S8"; break;
			case INS_TYPE_U16: o << ".U16"; break;
			case INS_TYPE_S16: o << ".S16"; break;
			case INS_TYPE_U32: o << ".U32"; break;
			case INS_TYPE_S32: o << ".S32"; break;
			case INS_TYPE_U64: o << ".U64"; break;
			case INS_TYPE_S64: o << ".S64"; break;
			case INS_TYPE_L: o << ".L"; break;
			case INS_TYPE_T: o << ".T"; break;
			case INS_TYPE_F32: o << ".F32"; break;
			case INS_TYPE_F64: o << ".F64"; break;
		}
		static const char PARAM_MODE_PREFIX[] = {' ', 'r', '$', '%'};
		if (i.md != INS_MODE_NONE) o << ' ' << PARAM_MODE_PREFIX[i.md] << i.pd;
		if (i.ma != INS_MODE_NONE) o << ' ' << PARAM_MODE_PREFIX[i.ma] << i.pa;
		if (i.mb != INS_MODE_NONE) o << ' ' << PARAM_MODE_PREFIX[i.mb] << i.pb;
		return o;
	}
};


struct LogicAND {
	uint8_t operator()(uint8_t a, uint8_t b) const { return a && b; }
	static constexpr const char *name = "and";
};

struct LogicOR {
	uint8_t operator()(uint8_t a, uint8_t b) const { return a || b; }
	static constexpr const char *name = "or";
};

struct LogicXOR {
	uint8_t operator()(uint8_t a, uint8_t b) const { return a != b; }
	static constexpr const char *name = "xor";
};

struct LogicNEG {
	uint8_t operator()(uint8_t a) const { return !a; }
	static constexpr const char *name = "neg";
};

class BasicInstruction {
public:
	virtual void execute(Process*) const = 0;
	virtual std::string describe() const = 0;

	std::tuple<uint8_t*,size_t> resolve_rval(Process *proc, uint16_t regid) const;
	std::tuple<uint8_t*,size_t> resolve_lval(Process *proc, uint16_t regid) const;
};

enum ProgramArgumentType {
	PROG_ARG_INVALID,
	PROG_ARG_SIGNED,
	PROG_ARG_UNSIGNED,
	PROG_ARG_TIME,
	PROG_ARG_LOGIC
};

class ProgramArgument {
public:
	ProgramArgumentType type = PROG_ARG_INVALID;
	unsigned length = 0;

	ProgramArgument() {}
	ProgramArgument(ProgramArgumentType type, unsigned length):
		type(type),
		length(length) {}
};

class ProgramRegister {
public:
	void *data;
	size_t length;

	ProgramRegister(void *data, size_t length): data(data), length(length) {}
};

class Program {
public:
	unsigned memory_size = 0;
	std::vector<Instruction> instructions;
	std::vector<std::unique_ptr<BasicInstruction>> instructions2;
	std::vector<ProgramRegister> constants;
	std::vector<uint8_t> constants_memory;
	std::vector<size_t> registers;
	std::vector<ProgramArgument> inputs;
	std::vector<ProgramArgument> outputs;

	unsigned alloc_memory(unsigned w) {
		auto m = memory_size;
		memory_size += w;
		return m;
	}

	void add_constant(void* data, size_t length) {
		constants.emplace_back(data, length);
		constants_memory.insert(constants_memory.end(), (uint8_t*)data, (uint8_t*)data + length);
		size_t l = 0;
		for (auto& c : constants) {
			c.data = &constants_memory[l];
			l += c.length;
		}
	}

	class InstructionBuilder {
		friend class Program;
		Instruction& ins;
		InstructionBuilder(Instruction& ins): ins(ins) {}
	public:
		#define SETTERS(X) \
		InstructionBuilder& r##X(uint64_t v) {\
			ins.m##X = INS_MODE_REG;\
			ins.p##X = v;\
			return *this;\
		}\
		template<typename T>\
		InstructionBuilder& i##X(T v) {\
			ins.m##X = INS_MODE_IMM;\
			*(T*)&ins.p##X = v;\
			return *this;\
		}\
		InstructionBuilder& m##X(uint64_t v) {\
			ins.m##X = INS_MODE_MEM;\
			ins.p##X = v;\
			return *this;\
		}

		SETTERS(d)
		SETTERS(a)
		SETTERS(b)
		#undef SETTERS
	};

	InstructionBuilder ins(uint16_t op, uint8_t type = INS_TYPE_NONE) {
		instructions.push_back(Instruction());
		auto& i = instructions.back();
		i.op = op;
		i.type = type;
		return InstructionBuilder(i);
	}
};

enum ProcessState {
	PROCESS_READY = 0,
	PROCESS_RUNNING = 1,
	PROCESS_SUSPENDED = 2,
	PROCESS_STOPPED = 3
};

class Process {
public:
	unsigned pc = 0;
	ProcessState state = PROCESS_READY;
	const Program *program;
	std::vector<void*> registers;
	std::vector<uint8_t> memory;
	std::vector<uint8_t> registers_memory;
	std::set<void*> sensitivity;
	std::vector<void*> inputs;
	std::vector<void*> outputs;

	Process() {}
	explicit Process(const Program *program):
		program(program),
		registers(program->registers.size()),
		inputs(program->inputs.size()),
		outputs(program->outputs.size()) {

		size_t lr = 0;
		for (auto& r : program->registers)
			lr += r;
		registers_memory.resize(lr);
		lr = 0;
		for (unsigned i = 0; i < registers.size(); i++) {
			registers[i] = &registers_memory[lr];
			lr += program->registers[i];
		}
	}

	void run() {
		if (state == PROCESS_STOPPED)
			return;
		state = PROCESS_RUNNING;
		while (state == PROCESS_RUNNING) {
			if (pc == program->instructions2.size()) {
				state = PROCESS_READY;
				pc = 0;
				break;
			}
			assert(pc < program->instructions2.size() && "pc jumped beyond end of program");

			// Instruction& ins = program->instructions[pc++];
			// run_ins(ins);
			std::cout << "  #" << pc;
			auto& ins = program->instructions2[pc++];
			std::cout << ": " << ins->describe() << '\n';
			ins->execute(this);
		}
	}

	void run_ins(Instruction& ins) {
		void *vd = nullptr, *va = nullptr, *vb = nullptr;
		switch (ins.md) {
		case INS_MODE_REG: vd = &memory[ins.pd]; break;
		case INS_MODE_IMM: vd = &ins.pd; break;
		case INS_MODE_MEM: vd = *(void**)&memory[ins.pd]; break;
		}
		switch (ins.ma) {
		case INS_MODE_REG: va = &memory[ins.pa]; break;
		case INS_MODE_IMM: va = &ins.pa; break;
		case INS_MODE_MEM: va = *(void**)&memory[ins.pa]; break;
		}
		switch (ins.mb) {
		case INS_MODE_REG: vb = &memory[ins.pb]; break;
		case INS_MODE_IMM: vb = &ins.pb; break;
		case INS_MODE_MEM: vb = *(void**)&memory[ins.pb]; break;
		}

		std::cout << ins << '\n';

		switch (ins.op & INS_MASK_GRP) {
		case INS_GRP_LD:
			switch (ins.op) {
			case INS_OP_LD:
				switch (ins.type) {
				#define CASE_LD(name, type) case INS_TYPE_##name:\
					*(type*)vd = *(type*)va; break;
				CASE_LD(U8, uint8_t);
				CASE_LD(S8, int8_t);
				CASE_LD(U16, uint16_t);
				CASE_LD(S16, int16_t);
				CASE_LD(U32, uint32_t);
				CASE_LD(S32, int32_t);
				CASE_LD(U64, uint64_t);
				CASE_LD(S64, int64_t);
				CASE_LD(F32, float);
				CASE_LD(F64, double);
				#undef CASE_LD
				default: goto ins_invalid;
				} break;
			case INS_OP_IN: {
				unsigned i = *(uint64_t*)va;
				assert(i < inputs.size());
				*(uint64_t*)vd = (uint64_t)inputs[i];
				} break;
			default: goto ins_invalid;
			} break;

		case INS_GRP_CMP:
			switch (ins.type) {
			#define CASE_CMP(name, type) case INS_TYPE_##name:\
				switch (ins.op) {\
				case INS_OP_CMPEQ:  *(uint8_t*)vd = (*(type*)va == *(type*)vb); break;\
				case INS_OP_CMPNEQ: *(uint8_t*)vd = (*(type*)va != *(type*)vb); break;\
				case INS_OP_CMPLT:  *(uint8_t*)vd = (*(type*)va <  *(type*)vb); break;\
				case INS_OP_CMPGT:  *(uint8_t*)vd = (*(type*)va >  *(type*)vb); break;\
				case INS_OP_CMPLEQ: *(uint8_t*)vd = (*(type*)va <= *(type*)vb); break;\
				case INS_OP_CMPGEQ: *(uint8_t*)vd = (*(type*)va >= *(type*)vb); break;\
				default: goto ins_invalid;\
				} break;
			CASE_CMP(U8, uint8_t);
			CASE_CMP(S8, int8_t);
			CASE_CMP(U16, uint16_t);
			CASE_CMP(S16, int16_t);
			CASE_CMP(U32, uint32_t);
			CASE_CMP(S32, int32_t);
			CASE_CMP(U64, uint64_t);
			CASE_CMP(S64, int64_t);
			CASE_CMP(F32, float);
			CASE_CMP(F64, double);
			#undef CASE_CMP
			default: goto ins_invalid;
			} break;

		case INS_GRP_BR:
			switch (ins.type) {
			#define CASE_BR(name, type) case INS_TYPE_##name:\
				switch (ins.op) {\
				case INS_OP_BR: pc = pc-1 + *(type*)va; break;\
				case INS_OP_BRC: if (*(uint8_t*)vd) pc = pc-1 + *(type*)va; break;\
				case INS_OP_BRCE: if (*(uint8_t*)vd) pc = pc-1 + *(type*)va; else pc = pc-1 + *(type*)vb; break;\
				default: goto ins_invalid;\
				} break;
			CASE_BR(U8, uint8_t);
			CASE_BR(S8, int8_t);
			CASE_BR(U16, uint16_t);
			CASE_BR(S16, int16_t);
			CASE_BR(U32, uint32_t);
			CASE_BR(S32, int32_t);
			CASE_BR(U64, uint64_t);
			CASE_BR(S64, int64_t);
			#undef CASE_BR
			default: goto ins_invalid;
			} break;

		case INS_GRP_ARI:
			switch (ins.type) {
			#define CASE_ARI(name, type) case INS_TYPE_##name:\
				switch (ins.op) {\
				case INS_OP_ADD:  *(type*)vd = (*(type*)va + *(type*)vb); break;\
				case INS_OP_SUB:  *(type*)vd = (*(type*)va - *(type*)vb); break;\
				case INS_OP_MUL:  *(type*)vd = (*(type*)va * *(type*)vb); break;\
				case INS_OP_DIV:  *(type*)vd = (*(type*)va / *(type*)vb); break;\
				default: goto ins_invalid;\
				} break;
			CASE_ARI(U8, uint8_t);
			CASE_ARI(S8, int8_t);
			CASE_ARI(U16, uint16_t);
			CASE_ARI(S16, int16_t);
			CASE_ARI(U32, uint32_t);
			CASE_ARI(S32, int32_t);
			CASE_ARI(U64, uint64_t);
			CASE_ARI(S64, int64_t);
			CASE_ARI(F32, float);
			CASE_ARI(F64, double);
			#undef CASE_ARI
			default: goto ins_invalid;
			} break;

		case INS_GRP_LOG:
			switch (ins.type) {
			#define CASE_LOG(name, type) case INS_TYPE_##name:\
				switch (ins.op) {\
				case INS_OP_NEG: *(type*)vd = ~(*(type*)va); break;\
				case INS_OP_AND: *(type*)vd = (*(type*)va & *(type*)vb); break;\
				case INS_OP_OR:  *(type*)vd = (*(type*)va | *(type*)vb); break;\
				case INS_OP_XOR: *(type*)vd = (*(type*)va ^ *(type*)vb); break;\
				default: goto ins_invalid;\
				} break;
			CASE_LOG(U8, uint8_t);
			CASE_LOG(S8, int8_t);
			CASE_LOG(U16, uint16_t);
			CASE_LOG(S16, int16_t);
			CASE_LOG(U32, uint32_t);
			CASE_LOG(S32, int32_t);
			CASE_LOG(U64, uint64_t);
			CASE_LOG(S64, int64_t);
			case INS_TYPE_L: {

			} break;
			#undef CASE_LOG
			default: goto ins_invalid;
			} break;

		// case INS_GRP_WAIT:
		// 	switch (ins.op) {
		// 	case INS_OP_WAITA:
		// 		if (!wait_time_set) {
		// 			wait_time_set = true;
		// 			wait_time = *(uint64_t*)va;
		// 		}
		// 		if (*input_time < wait_time) {
		// 			--pc;
		// 			state = PROCESS_SUSPENDED;
		// 		} else {
		// 			wait_time_set = false;
		// 		}
		// 		break;
		// 	case INS_OP_WAITR:
		// 		if (!wait_time_set) {
		// 			wait_time_set = true;
		// 			wait_time = *input_time + *(uint64_t*)va;
		// 		}
		// 		if (*input_time < wait_time) {
		// 			--pc;
		// 			state = PROCESS_SUSPENDED;
		// 		} else {
		// 			wait_time_set = false;
		// 		}
		// 		break;
		// 	case INS_OP_WAITW:
		// 		--pc;
		// 		state = PROCESS_SUSPENDED;
		// 		break;
		// 	default: goto ins_invalid;
		// 	} break;

		case INS_GRP_DBG:
			std::cout << "[PROC " << this << ", pc=" << pc-1 << "] ";

			auto f = std::cout.flags();
			switch (ins.ma) {
			case INS_MODE_REG:
				std::cout << 'r' << ins.pa << " = ";
				break;
			case INS_MODE_MEM:
				std::cout << '%' << std::hex << ins.pa << " = ";
				break;
			default: goto ins_invalid;
			}
			std::cout.flags(f);

			switch (ins.type) {
			case INS_TYPE_U8: std::cout  << (uint64_t)*(uint8_t*)va;  break;
			case INS_TYPE_S8: std::cout  <<  (int64_t)*(int8_t*)va;   break;
			case INS_TYPE_U16: std::cout << (uint64_t)*(uint16_t*)va; break;
			case INS_TYPE_S16: std::cout <<  (int64_t)*(int16_t*)va;  break;
			case INS_TYPE_U32: std::cout << (uint64_t)*(uint32_t*)va; break;
			case INS_TYPE_S32: std::cout <<  (int64_t)*(int32_t*)va;  break;
			case INS_TYPE_U64: std::cout << (uint64_t)*(uint64_t*)va; break;
			case INS_TYPE_S64: std::cout <<  (int64_t)*(int64_t*)va;  break;
			case INS_TYPE_F32: std::cout << *(float*)va; break;
			case INS_TYPE_F64: std::cout << *(double*)va; break;
			default: goto ins_invalid;
			}
			std::cout << '\n';
			break;

		// case INS_OP_LD:     vd = va; break;
		// case INS_OP_CMPEQ:  vd = (va == vb); break;
		// case INS_OP_CMPNEQ: vd = (va != vb); break;
		// case INS_OP_CMPLT:  vd = (va < vb); break;
		// case INS_OP_CMPGT:  vd = (va > vb); break;
		// case INS_OP_CMPLEQ: vd = (va <= vb); break;
		// case INS_OP_CMPGEQ: vd = (va >= vb); break;
		// case INS_OP_BR:     pc = va; break;
		// case INS_OP_BRC:    if (vb) pc = va; break;
		// case INS_OP_BRCE:   if (vb) pc = va; break;
		// case INS_OP_ADD:    vd = va + vb; break;
		// case INS_OP_SUB:    vd = va - vb; break;
		// case INS_OP_MUL:    vd = va * vb; break;
		// case INS_OP_DIV:    vd = va / vb; break;
		// case INS_OP_MOD:    vd = va % vb; break;
		// case INS_OP_AND:    vd = va & vb; break;
		// case INS_OP_NAND:   vd = ~(va & vb); break;
		// case INS_OP_OR:     vd = va | vb; break;
		// case INS_OP_NOR:    vd = ~(va | vb); break;
		// case INS_OP_XOR:    vd = va ^ vb; break;
		// case INS_OP_EQV:    vd = ~(va ^ vb); break;
		}

		return;
	ins_invalid:
		state = PROCESS_STOPPED;
		std::cerr << "invalid instruction: " << ins << '\n';
		abort();
	}

	void run_ins_logic_bin(unsigned op, unsigned num, uint8_t *rd, uint8_t const *ra, uint8_t const *rb) {
		for (unsigned i = 0; i < num; ++i) {
			auto& vd = rd[i];
			auto& va = ra[i];
			auto& vb = rb[i];
			unsigned ia, ib, id;

			if (va == '0' || va == 'L') {
				ia = 0;
			} else if (va == '1' || va == 'H') {
				ia = 1;
			} else {
				vd = 'X';
				continue;
			}

			if (vb == '0' || vb == 'L') {
				ib = 0;
			} else if (vb == '1' || vb == 'H') {
				ib = 1;
			} else {
				vd = 'X';
				continue;
			}

			switch (op) {
				case INS_OP_AND: id = (ia && ib ? 1 : 0); break;
				case INS_OP_OR:  id = (ia || ib ? 1 : 0); break;
				case INS_OP_XOR: id = (ia != ib ? 1 : 0); break;
			}

			vd = (id == 0 ? '0' : '1');
		}
	}
};

std::tuple<uint8_t*,size_t> BasicInstruction::resolve_rval(Process *proc, uint16_t regid) const {
	if (regid & 0x8000) {
		auto i = regid & ~0x8000;
		assert(i < proc->program->constants.size());
		return std::make_tuple(
			(uint8_t*)proc->program->constants[i].data,
			(size_t)proc->program->constants[i].length);
	} else {
		return resolve_lval(proc,regid);
	}
}

std::tuple<uint8_t*,size_t> BasicInstruction::resolve_lval(Process *proc, uint16_t regid) const {
	assert(regid < proc->registers.size());
	return std::make_tuple(
		(uint8_t*)proc->registers[regid],
		(size_t)proc->program->registers[regid]);
}

class InputInstruction : public BasicInstruction {
public:
	uint16_t rd;
	uint16_t input;

	InputInstruction(uint16_t rd, uint16_t input):
		rd(rd), input(input) {}

	void execute(Process *proc) const {
		assert(proc);
		assert(rd < proc->registers.size());
		assert(input < proc->inputs.size());

		auto len = proc->program->registers[rd];
		assert(len == proc->program->inputs[input].length);
		uint8_t *src = (uint8_t*)proc->inputs[input];
		uint8_t *dst = (uint8_t*)proc->registers[rd];
		std::copy(src, src+len, dst);
	}

	std::string describe() const {
		return "in r" + std::to_string(rd) + " " + std::to_string(input);
	}
};

class OutputInstruction : public BasicInstruction {
public:
	uint16_t output;
	uint16_t ra;

	OutputInstruction(uint16_t output, uint16_t ra):
		output(output), ra(ra) {}

	void execute(Process *proc) const {
		assert(proc);
		assert(output < proc->outputs.size());

		uint8_t *pa;
		size_t lena;
		std::tie(pa,lena) = resolve_rval(proc,ra);

		assert(lena == proc->program->outputs[output].length);
		std::copy(pa, pa+lena, (uint8_t*)proc->outputs[output]);
	}

	std::string describe() const {
		return "out " + std::to_string(output) + " r" + std::to_string(ra);
	}
};

class MoveInstruction : public BasicInstruction {
public:
	uint16_t rd;
	uint16_t ra;

	MoveInstruction(uint16_t rd, uint16_t ra):
		rd(rd), ra(ra) {}

	void execute(Process *proc) const {
		assert(proc);

		uint8_t *pd, *pa;
		size_t lend, lena;
		std::tie(pd,lend) = resolve_lval(proc,rd);
		std::tie(pa,lena) = resolve_lval(proc,ra);
		assert(lend == lena);

		std::copy(pa, pa+lena, pd);
	}

	std::string describe() const {
		return "mov r" + std::to_string(rd) + " r" + std::to_string(ra);
	}
};

template<class Operation>
class UnaryLogicInstruction : public BasicInstruction {
public:
	unsigned num;
	uint16_t rd;
	uint16_t ra;

	UnaryLogicInstruction(unsigned num, uint16_t rd, uint16_t ra):
		num(num), rd(rd), ra(ra) {}

	void execute(Process *proc) const {
		assert(proc);

		uint8_t *pd, *pa;
		size_t lend, lena;
		std::tie(pd, lend) = resolve_lval(proc, rd);
		std::tie(pa, lena) = resolve_rval(proc, ra);

		assert(lend == lena);
		assert(num == lena);

		for (unsigned i = 0; i < num; ++i) {
			auto& vd = pd[i];
			auto& va = pa[i];
			uint8_t ia, id;

			if (va == '0' || va == 'L') {
				ia = 0;
			} else if (va == '1' || va == 'H') {
				ia = 1;
			} else {
				vd = 'X';
				continue;
			}

			id = Operation()(ia);
			vd = (id == 0 ? '0' : '1');
		}
	}

	std::string describe() const {
		return "log." + std::string(Operation::name) +
			" " + std::to_string(num) +
			" r" + std::to_string(rd) +
			" r" + std::to_string(ra);
	}
};

template<class Operation>
class BinaryLogicInstruction : public BasicInstruction {
public:
	unsigned num;
	uint16_t rd;
	uint16_t ra;
	uint16_t rb;

	BinaryLogicInstruction(unsigned num, uint16_t rd, uint16_t ra, uint16_t rb):
		num(num), rd(rd), ra(ra), rb(rb) {}

	void execute(Process *proc) const {
		assert(proc);

		uint8_t *pd, *pa, *pb;
		size_t lend, lena, lenb;
		std::tie(pd, lend) = resolve_lval(proc, rd);
		std::tie(pa, lena) = resolve_rval(proc, ra);
		std::tie(pb, lenb) = resolve_rval(proc, rb);

		assert(lend == lena);
		assert(lend == lenb);
		assert(num == lena);

		for (unsigned i = 0; i < num; ++i) {
			auto& vd = pd[i];
			auto& va = pa[i];
			auto& vb = pb[i];
			uint8_t ia, ib, id;

			if (va == '0' || va == 'L') {
				ia = 0;
			} else if (va == '1' || va == 'H') {
				ia = 1;
			} else {
				vd = 'X';
				continue;
			}

			if (vb == '0' || vb == 'L') {
				ib = 0;
			} else if (vb == '1' || vb == 'H') {
				ib = 1;
			} else {
				vd = 'X';
				continue;
			}

			id = Operation()(ia,ib);
			vd = (id == 0 ? '0' : '1');
		}
	}

	std::string describe() const {
		return "log." + std::string(Operation::name) +
			" " + std::to_string(num) +
			" r" + std::to_string(rd) +
			" r" + std::to_string(ra) +
			" r" + std::to_string(rb);
	}
};

struct ArithmeticAdd {
	void operator()(
		unsigned len, uint64_t *pd, uint64_t const* pa, uint64_t const* pb) {
		assert(len < 64 && "more than 64bit not yet supported");
		*pd = *pa + *pb;
	}
	static constexpr const char *name = "add";
};

struct ArithmeticSubtract {
	void operator()(
		unsigned len, uint64_t *pd, uint64_t const* pa, uint64_t const* pb) {
		assert(len < 64 && "more than 64bit not yet supported");
		*pd = *pa - *pb;
	}
	static constexpr const char *name = "sub";
};

struct ArithmeticMultiply {
	void operator()(
		unsigned len, uint64_t *pd, uint64_t const* pa, uint64_t const* pb) {
		assert(len < 64 && "more than 64bit not yet supported");
		*pd = *pa * *pb;
	}
	static constexpr const char *name = "mul";
};

struct ArithmeticDivide {
	void operator()(
		unsigned len, uint64_t *pd, uint64_t const* pa, uint64_t const* pb) {
		assert(len < 64 && "more than 64bit not yet supported");
		*pd = *pa / *pb;
	}
	static constexpr const char *name = "div";
};

template<class Operation>
class BinaryArithmeticLogicInstruction : public BasicInstruction {
public:
	unsigned num;
	uint16_t rd;
	uint16_t ra;
	uint16_t rb;

	BinaryArithmeticLogicInstruction(
		unsigned num, uint16_t rd, uint16_t ra, uint16_t rb):
		num(num), rd(rd), ra(ra), rb(rb) {}

	void execute(Process *proc) const {
		assert(proc);

		uint8_t *pd, *pa, *pb;
		size_t lend, lena, lenb;
		std::tie(pd, lend) = resolve_lval(proc, rd);
		std::tie(pa, lena) = resolve_rval(proc, ra);
		std::tie(pb, lenb) = resolve_rval(proc, rb);

		assert(lend == lena);
		assert(lend == lenb);
		assert(num == lena);

		unsigned num_chunks = (lena+63)/64;
		uint64_t id[num_chunks];
		uint64_t ia[num_chunks];
		uint64_t ib[num_chunks];

		std::fill(ia, ia+num_chunks, 0);
		std::fill(ib, ib+num_chunks, 0);

		for (unsigned i = 0; i < num; ++i) {
			unsigned nc = (num-i-1) / 64;
			unsigned nb = (num-i-1) % 64;
			uint64_t mask = 1 << (uint64_t)nb;

			auto& va = pa[i];
			auto& vb = pb[i];

			if (va == '0' || va == 'L') {
				ia[nc] &= ~mask;
			} else if (va == '1' || va == 'H') {
				ia[nc] |= mask;
			} else {
				std::fill(pd, pd+num, 'X');
				return;
			}

			if (vb == '0' || vb == 'L') {
				ib[nc] &= ~mask;
			} else if (vb == '1' || vb == 'H') {
				ib[nc] |= mask;
			} else {
				std::fill(pd, pd+num, 'X');
				return;
			}
		}

		Operation()(num, id, ia, ib);

		for (unsigned i = 0; i < num; ++i) {
			unsigned nc = (num-i-1) / 64;
			unsigned nb = (num-i-1) % 64;
			uint64_t mask = 1 << (uint64_t)nb;
			pd[i] = (id[nc] & mask ? '1' : '0');
		}
	}

	std::string describe() const {
		return "log." + std::string(Operation::name) +
			" " + std::to_string(num) +
			" r" + std::to_string(rd) +
			" r" + std::to_string(ra) +
			" r" + std::to_string(rb);
	}
};

int main(int argc, char** argv) {

	uint64_t T = 12563;
	char allone[] = "10101010";
	char one[] = "00000001";
	char three[] = "00000011";
	char addr[] = "00000000";

	Program program;
	program.inputs.emplace_back(PROG_ARG_TIME, sizeof(T));
	program.inputs.emplace_back(PROG_ARG_LOGIC, 8);
	program.outputs.emplace_back(PROG_ARG_LOGIC, 8);
	program.registers.push_back(8);
	program.registers.push_back(8);
	program.registers.push_back(8);
	program.add_constant(allone,8);
	program.add_constant(one,8);
	program.add_constant(three,8);

	program.instructions2.emplace_back(new InputInstruction(0,1));
	program.instructions2.emplace_back(new MoveInstruction(1,0));
	program.instructions2.emplace_back(new MoveInstruction(2,0));
	// program.instructions2.emplace_back(new UnaryLogicInstruction<LogicNEG>(8,0,0));
	// program.instructions2.emplace_back(new BinaryLogicInstruction<LogicXOR>(8,1,1,0x8000));
	program.instructions2.emplace_back(new BinaryArithmeticLogicInstruction<ArithmeticAdd>(8,0,2,0x8000|1));
	program.instructions2.emplace_back(new BinaryArithmeticLogicInstruction<ArithmeticMultiply>(8,1,2,0x8000|2));
	program.instructions2.emplace_back(new OutputInstruction(0,0));

	// auto mT = program.alloc_memory(sizeof(&T));
	// auto rstuff = program.alloc_memory(8);
	// auto rloop = program.alloc_memory(1);
	// auto rexit = program.alloc_memory(1);
	// program.ins(INS_OP_IN).rd(rstuff).ia(0);
	// program.ins(INS_OP_LD, INS_TYPE_U64).rd(rstuff).ma(rstuff);
	// program.ins(INS_OP_DBG, INS_TYPE_U64).ra(rstuff);
	// program.ins(INS_OP_LD, INS_TYPE_U8).rd(rloop).ia(4);
	// program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rloop);
	// program.ins(INS_OP_DBG, INS_TYPE_U64).ma(mT);
	// program.ins(INS_OP_SUB, INS_TYPE_U8).rd(rloop).ra(rloop).ib(1);
	// program.ins(INS_OP_WAITR).ia(3);
	// program.ins(INS_OP_CMPNEQ, INS_TYPE_U8).rd(rexit).ra(rloop).ib(0);
	// program.ins(INS_OP_BRC, INS_TYPE_S8).rd(rexit).ia(-2);
	// program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rloop);
	// program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rexit);

	Process process(&program);
	// process.program = &program;
	// process.inputs.resize(program.inputs.size());
	// process.outputs.resize(program.outputs.size());
	process.inputs[0] = &T;
	process.inputs[1] = addr;
	process.outputs[0] = addr;
	// process.memory.resize(program.memory_size);
	// *(uint64_t**)&process.memory[mT] = &T;

	std::cout << "Process:\n";
	std::cout << "  registers: " << process.registers.size() << '\n';
	std::cout << "  registers memory: " << process.registers_memory.size() << " Bytes\n";

	std::vector<Process*> processes = {&process};
	for (T = 0; T <= 10; ++T) {
		std::cout << "[SIM " << T << "]\n";
		for (auto p : processes) {
			p->run();
			std::cout << "  r0 = " << std::string((const char*)process.registers[0], 8) << '\n';
			std::cout << "  r1 = " << std::string((const char*)process.registers[1], 8) << '\n';
			std::cout << "  r2 = " << std::string((const char*)process.registers[2], 8) << '\n';
		}
	}

	return 0;
}
