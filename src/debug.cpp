/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/common.hpp"
#include "llhd/ir/module.hpp"
#include "llhd/ir/process.hpp"
#include "llhd/ir/argument.hpp"
#include "llhd/ir/basicblock.hpp"
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/context.hpp"
#include "llhd/ir/constants.hpp"

using namespace llhd;

// TODO:
// - arbitrary-precision integers
// - logic values, then use them inside ConstantLogic
// - introduce opcodes for Instruction class
// - introduce id for Value class (analogous to Type)

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
	Argument * Acarry = new Argument("carry", Type::getLogicType(C,1), P);
	Argument * Aflag = new Argument("flag", Type::getLogicType(C,1), P);
	Argument * Aresult = new Argument("result", Type::getLogicType(C,4), P);
	P->getInputList().push_back(Adata_a);
	P->getInputList().push_back(Adata_b);
	P->getInputList().push_back(Aoperation);
	P->getOutputList().push_back(Acarry);
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
	I->insertAtEnd(BB);

	// case operation is
	SwitchInst * SI = new SwitchInst(Aoperation, BBothers);
	SI->insertAtEnd(BB);
	SI->addDestination(Value::getConst(Aoperation->getType(), "000"), BB000);
	SI->addDestination(Value::getConst(Aoperation->getType(), "001"), BB001);
	SI->addDestination(Value::getConst(Aoperation->getType(), "010"), BB010);
	SI->addDestination(Value::getConst(Aoperation->getType(), "011"), BB011);
	SI->addDestination(Value::getConst(Aoperation->getType(), "100"), BB100);
	SI->addDestination(Value::getConst(Aoperation->getType(), "101"), BB101);
	SI->addDestination(Value::getConst(Aoperation->getType(), "110"), BB110);

	// when "000" =>
	// temp <= std_logic_vector(unsigned("0" & data_a) + unsigned(data_b))
	Instruction * IA = new InsertValueInst(Value::getConstNull(Type::getLogicType(C,5)), Value::getConstNull(Type::getLogicType(C,1)), ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	IA->insertAtEnd(BB000);
	IA = new InsertValueInst(IA, Adata_a, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	IA->insertAtEnd(BB000);
	Instruction * IB = new InsertValueInst(Value::getConstNull(Type::getLogicType(C,5)), Value::getConstNull(Type::getLogicType(C,1)), ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	IB->insertAtEnd(BB000);
	IB = new InsertValueInst(IB, Adata_b, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	IB->insertAtEnd(BB000);
	I = new BinaryInst(Instruction::Add, IA, IB);
	I->insertAtEnd(BB000);
	Value * temp = I;

	// result <= temp(3 downto 0)
	I = new ExtractValueInst(temp, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	I->insertAtEnd(BB000);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB000);

	// carry <= temp(4)
	I = new ExtractValueInst(temp, ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	I->insertAtEnd(BB000);
	I = new DriveInst(Acarry, I);
	I->insertAtEnd(BB000);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB000);

	// when "001" =>
	// if (unsigned(data_a) >= unsigned(data_b))
	I = new CompareInst(CompareInst::UGE, Adata_a, Adata_b);
	I->insertAtEnd(BB001);
	BasicBlock * BB001A = new BasicBlock(C, "op001_ge", P);
	BasicBlock * BB001B = new BasicBlock(C, "op001_lt", P);
	P->getBasicBlockList().push_back(BB001A);
	P->getBasicBlockList().push_back(BB001B);
	I = new BranchInst(BB001A, BB001B, I);
	I->insertAtEnd(BB001);

	// -- if true
	// result <= std_logic_vector(unsigned(data_a) - unsigned(data_b))
	I = new BinaryInst(Instruction::Sub, Adata_a, Adata_b);
	I->insertAtEnd(BB001A);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB001A);

	// flag <= '0'
	I = new DriveInst(Aflag, Value::getConst(Aflag->getType(), "0"));
	I->insertAtEnd(BB001A);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB001A);

	// -- if false
	// result <= std_logic_vector(unsigned(data_b) - unsigned(data_a))
	I = new BinaryInst(Instruction::Sub, Adata_b, Adata_a);
	I->insertAtEnd(BB001B);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB001B);

	// flag <= '0'
	I = new DriveInst(Aflag, Value::getConst(Aflag->getType(), "0"));
	I->insertAtEnd(BB001B);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB001B);

	// when "010" =>
	// result <= data_a and data_b
	I = new BinaryInst(Instruction::And, Adata_a, Adata_b);
	I->insertAtEnd(BB010);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB010);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB010);

	// when "011" =>
	// result <= data_a or data_b
	I = new BinaryInst(Instruction::Or, Adata_a, Adata_b);
	I->insertAtEnd(BB011);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB011);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB011);

	// when "100" =>
	// result <= data_a xor data_b
	I = new BinaryInst(Instruction::Xor, Adata_a, Adata_b);
	I->insertAtEnd(BB100);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB100);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB100);

	// when "101" =>
	// result <= not data_a
	I = new BinaryInst(Instruction::Xor, Adata_a, Value::getConst(Adata_a->getType(), "1111"));
	I->insertAtEnd(BB101);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB101);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB101);

	// when "110" =>
	// result <= not data_b
	I = new BinaryInst(Instruction::Xor, Adata_b, Value::getConst(Adata_b->getType(), "1111"));
	I->insertAtEnd(BB101);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BB101);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BB101);

	// when others =>
	// temp <= std_logic_vector(unsigned("0" & data_a) + unsigned(not data_b) + 1)
	IA = new InsertValueInst(Value::getConstNull(Type::getLogicType(C,5)), Value::getConstNull(Type::getLogicType(C,1)), ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	IA->insertAtEnd(BBothers);
	IA = new InsertValueInst(IA, Adata_a, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	IA->insertAtEnd(BBothers);
	I = new BinaryInst(Instruction::Xor, Adata_b, Value::getConst(Adata_b->getType(), "1111"));
	I->insertAtEnd(BBothers);
	IB = new InsertValueInst(Value::getConstNull(Type::getLogicType(C,5)), Value::getConstNull(Type::getLogicType(C,1)), ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	IB->insertAtEnd(BBothers);
	IB = new InsertValueInst(IB, I, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	IB->insertAtEnd(BBothers);
	I = new BinaryInst(Instruction::Add, IA, IB);
	I->insertAtEnd(BBothers);
	I = new BinaryInst(Instruction::Add, I, Value::getConst(Type::getLogicType(C,5), "00001"));
	I->insertAtEnd(BBothers);
	temp = I;

	// result <= temp(3 downto 0)
	I = new ExtractValueInst(temp, ConstantInteger::get(IntegerType::get(C,1), 0), 4);
	I->insertAtEnd(BBothers);
	I = new DriveInst(Aresult, I);
	I->insertAtEnd(BBothers);

	// flag <= temp(4)
	I = new ExtractValueInst(temp, ConstantInteger::get(IntegerType::get(C,3), 4), 1);
	I->insertAtEnd(BBothers);
	I = new DriveInst(Aflag, I);
	I->insertAtEnd(BBothers);

	I = new BranchInst(BBexit, nullptr, nullptr);
	I->insertAtEnd(BBothers);

	return 0;
}
