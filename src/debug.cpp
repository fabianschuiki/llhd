/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/common.hpp"
#include "llhd/ir/module.hpp"
#include "llhd/ir/process.hpp"
#include "llhd/ir/argument.hpp"
#include "llhd/ir/basicblock.hpp"
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/context.hpp"

using namespace llhd;

// entity is parallel and driving
// process is sequential and driving
// function is sequential and immediate

// types and certain values are managed in a context

// static get implies the returned ptr is memory-managed
// static make implies the caller is responsible for deleting the returned ptr

int main() {
	tfm::format(std::cout, "llhd version %s\n", VERSION);

	// build alu module
	// apply stimuli and record the reaction

	Context C;
	Module * M = new Module;
	Process * P = new Process("alu", M);
	Argument * Adata_a = new Argument("data_a", Type::getLogicType(C,4), P);
	Argument * Adata_b = new Argument("data_b", Type::getLogicType(C,4), P);
	Argument * Aoperation = new Argument("operation", Type::getLogicType(C,3), P);
	Argument * Acarry_out = new Argument("carry_out", Type::getLogicType(C,1), P);
	Argument * Aflag = new Argument("flag", Type::getLogicType(C,1), P);
	Argument * Aresult = new Argument("result", Type::getLogicType(C,4), P);
	P->getInputList().push_back(Adata_a);
	P->getInputList().push_back(Adata_b);
	P->getInputList().push_back(Aoperation);
	P->getOutputList().push_back(Acarry_out);
	P->getOutputList().push_back(Aflag);
	P->getOutputList().push_back(Aresult);

	BasicBlock * BB = new BasicBlock(C, "entry", P);
	BasicBlock * BB000 = new BasicBlock(C, "op000", P);
	BasicBlock * BB001 = new BasicBlock(C, "op001", P);
	BasicBlock * BB010 = new BasicBlock(C, "op010", P);
	BasicBlock * BB011 = new BasicBlock(C, "op011", P);
	BasicBlock * BB100 = new BasicBlock(C, "op100", P);
	BasicBlock * BB101 = new BasicBlock(C, "op101", P);
	BasicBlock * BB110 = new BasicBlock(C, "op110", P);
	BasicBlock * BBothers = new BasicBlock(C, "others", P);
	BasicBlock * BBexit = new BasicBlock(C, "exit", P);
	P->getBasicBlockList().push_back(BB);
	P->getBasicBlockList().push_back(BB000);
	P->getBasicBlockList().push_back(BB001);
	P->getBasicBlockList().push_back(BB010);
	P->getBasicBlockList().push_back(BB011);
	P->getBasicBlockList().push_back(BB100);
	P->getBasicBlockList().push_back(BB100);
	P->getBasicBlockList().push_back(BB101);
	P->getBasicBlockList().push_back(BB110);
	P->getBasicBlockList().push_back(BBothers);
	P->getBasicBlockList().push_back(BBexit);

	// flag <= '0'
	Instruction * I = new DriveInst(Aflag, Value::getConstNull(Aflag->getType()));
	BB->getInstList().push_back(I);

	// SwitchInst * SI = new SwitchInst();

	// add inputs and outputs
	// add internal temporary signal
	// add process that handles behaviour
	// add inputs

	return 0;
}
