/* Copyright (c) 2015 Fabian Schuiki */
#include <cassert>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

enum InsOp {
	INS_MASK_GRP = 0xFF00,
	INS_MASK_OP  = 0x00FF,

	INS_GRP_LD   = 0x100,
	INS_GRP_CMP  = 0x200,
	INS_GRP_BR   = 0x300,
	INS_GRP_ARI  = 0x400,
	INS_GRP_LOG  = 0x500,
	INS_GRP_DBG  = 0x600,

	INS_OP_LD     = 0x100,

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

	INS_OP_DBG    = 0x600,
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

class Program {
public:
	unsigned memory_size = 0;
	std::vector<Instruction> instructions;

	unsigned alloc_memory(unsigned w) {
		auto m = memory_size;
		memory_size += w;
		return m;
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
	Program* program;
	std::vector<unsigned> registers;
	std::vector<char> memory;

	void run() {
		if (state == PROCESS_STOPPED)
			return;
		state = PROCESS_RUNNING;
		while (state == PROCESS_RUNNING) {
			if (pc == program->instructions.size()) {
				state = PROCESS_READY;
				pc = 0;
				break;
			}
			assert(pc < program->instructions.size() && "pc jumped beyond end of program");

			Instruction& ins = program->instructions[pc++];
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
				switch (ins.type) {
				#define CASE_LD(name, type) case INS_TYPE_##name:\
					switch (ins.op) {\
					case INS_OP_LD: *(type*)vd = *(type*)va; break;\
					default: goto ins_invalid;\
					} break;
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
				#undef CASE_LOG
				default: goto ins_invalid;
				} break;

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

			continue;
		ins_invalid:
			state = PROCESS_STOPPED;
			std::cerr << "invalid instruction: " << ins << '\n';
			abort();
		}
	}
};

int main(int argc, char** argv) {

	Program program;
	auto rloop = program.alloc_memory(1);
	auto rexit = program.alloc_memory(1);
	program.ins(INS_OP_LD, INS_TYPE_U8).rd(rloop).ia(4);
	program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rloop);
	program.ins(INS_OP_SUB, INS_TYPE_U8).rd(rloop).ra(rloop).ib(1);
	program.ins(INS_OP_CMPNEQ, INS_TYPE_U8).rd(rexit).ra(rloop).ib(0);
	program.ins(INS_OP_BRC, INS_TYPE_S8).rd(rexit).ia(-2);
	program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rloop);
	program.ins(INS_OP_DBG, INS_TYPE_U8).ra(rexit);

	Process process;
	process.program = &program;
	process.memory.resize(program.memory_size);

	process.run();

	return 0;
}
