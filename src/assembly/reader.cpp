/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/assembly/assembly.hpp"
#include "llhd/assembly/function.hpp"
#include "llhd/assembly/instruction.hpp"
#include "llhd/assembly/lexer.hpp"
#include "llhd/assembly/module.hpp"
#include "llhd/assembly/process.hpp"
#include "llhd/assembly/reader.hpp"
#include "llhd/assembly/time.hpp"
#include "llhd/assembly/type.hpp"
#include "llhd/assembly/value.hpp"
#include "llhd/diagnostic/diagnostic.hpp"
#include "llhd/utils/memory.hpp"
#include <iostream>

/// \file
/// \refactor The code snippets that emit diagnostic messages are highly
/// repetitive and can easily be generalized either as a function, or
/// preferrably as a builder class that could also be used elsewhere in the
/// code. Especially message formatting would be useful.

namespace llhd {

struct AssemblyReaderInternal {
	AssemblyLexer &m_input;
	DiagnosticContext *m_dctx;

	AssemblyReaderInternal(AssemblyLexer &input, DiagnosticContext *dctx)
		:	m_input(input), m_dctx(dctx) {
	}


	/// Parses the top level of the input until an end-of-file is encountered.
	/// The input lexer is expected to be located on the SOF token. That is,
	/// no call to `next()` should have been performed.
	/// \code{.ebnf}
	/// root      := root_stmt*
	/// root_stmt := module_def | process_def | function_def
	/// \endcode
	bool parse_root(Assembly &assembly) {
		m_input.next();
		while (m_input) {
			switch (m_input.current_token()) {
				case TOKEN_EOF: break;
				case TOKEN_KW_MOD: {
					Module mod;
					if (!parse_module(mod))
						return false;
					assembly.statements.push_back(std::move(mod));
				} break;
				case TOKEN_KW_PROC: {
					Process proc;
					if (!parse_process(proc))
						return false;
					assembly.statements.push_back(std::move(proc));
				} break;
				case TOKEN_KW_FUNC: {
					Function func;
					if (!parse_function(func))
						return false;
					assembly.statements.push_back(std::move(func));
				} break;
				default: {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected module, process, or function");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				} break;
			}
		}
		return true;
	}


	/// Parses an entire module instruction. Expects the input lexer to sit on a
	/// module keyword token.
	/// \code{.ebnf}
	/// module_def := "mod" name
	///               "(" input'module_args? ")"
	///               "(" output'module_args? ")"
	///               "{" module_body "}"
	/// \endcode
	bool parse_module(Module &mod) {
		assert(m_input.current_token() == TOKEN_KW_MOD);
		SourceRange mod_range = m_input.current_range();
		m_input.next();

		std::string name;
		mod_range = union_range(mod_range, m_input.current_range());
		if (!parse_name(name))
			return false;

		mod.name = name;

		// input arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected input arguments of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(mod.inputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after input arguments of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// output arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected output arguments of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(mod.outputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after output arguments of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// module body
		if (m_input.current_token() != TOKEN_LBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected body of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (!parse_body(mod.instructions))
				return false;
		}

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing braces after body of module '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(mod_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		return true;
	}


	/// Parses a list of comma-separated module input or output arguments.
	/// Expects the input lexer to sit on the first token of the first module.
	/// Expects at least one argument.
	/// \code{.ebnf}
	/// module_args := module_arg ("," module_arg)*
	/// \endcode
	template <class T>
	bool parse_args(std::vector<T> &args) {
		while (m_input) {
			T arg;
			if (!parse_arg(arg))
				return false;
			args.push_back(std::move(arg));
			if (m_input.current_token() == TOKEN_COMMA)
				m_input.next();
			else
				break;
		}
		return true;
	}


	/// Parses a single module input or output argument. Expects the input lexer
	/// to sit on the first token of the argument.
	/// \code{.ebnf}
	/// module_arg  := type local_name
	/// \endcode
	template <class T>
	bool parse_arg(T &arg) {
		if (m_input.current_token() != TOKEN_TYPE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected argument type");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		arg.type = UnknownType(m_input.current_string());
		m_input.next();

		if (m_input.current_token() != TOKEN_NAME_LOCAL) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected argument name (a local name)");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		arg.name = m_input.current_string();
		m_input.next();

		return true;
	}


	/// Parses an entire module body. Expects the input lexer to sit on the
	/// first token of the first instruction.
	/// \code{.ebnf}
	/// module_body := module_ins+
	/// module_ins  := pure_ins | stateful_ins
	/// \endcode
	bool parse_body(std::vector<Instruction> &instructions) {
		while (m_input && m_input.current_token() != TOKEN_RBRACE) {
			Instruction ins;
			if (!parse_instruction(ins))
				return false;
			assert(ins);
			instructions.push_back(std::move(ins));
		}
		return true;
	}

	/// Parses an entire process instruction. Expects the input lexer to sit on
	/// the proc keyword.
	/// \code{.ebnf}
	/// process_def := "proc" name
	///                "(" input'process_args? ")"
	///                "(" output'process_args? ")"
	///                "{" process_body "}"
	/// \endcode
	bool parse_process(Process &proc) {
		assert(m_input.current_token() == TOKEN_KW_PROC);
		SourceRange proc_range = m_input.current_range();
		m_input.next();

		std::string name;
		proc_range = llhd::union_range(proc_range, m_input.current_range());
		if (!parse_name(name))
			return false;

		proc.name = name;

		// input arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected input arguments of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(proc.inputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after input arguments of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// output arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected output arguments of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(proc.outputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after output arguments of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// process body
		if (m_input.current_token() != TOKEN_LBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected body of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (!parse_body(proc.instructions))
				return false;
		}

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing braces after body of process '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(proc_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		return true;
	}


	bool parse_function(Function &func) {
		assert(m_input.current_token() == TOKEN_KW_FUNC);
		SourceRange func_range = m_input.current_range();
		m_input.next();

		std::string name;
		func_range = llhd::union_range(func_range, m_input.current_range());
		if (!parse_name(name))
			return false;

		func.name = name;

		// input arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected input arguments of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(func.inputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after input arguments of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// output arguments
		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected output arguments of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_args(func.outputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after output arguments of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		// function body
		if (m_input.current_token() != TOKEN_LBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected body of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (!parse_body(func.instructions))
				return false;
		}

		if (m_input.current_token() != TOKEN_RBRACE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing braces after body of function '"+name+"'");
				msg->set_main_range(m_input.current_range());
				msg->add_highlit_range(func_range);
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		return true;
	}


	bool parse_name(std::string &name) {
		if (m_input.current_token() == TOKEN_NAME_LOCAL) {
			name = m_input.current_string();
			m_input.next();
			return true;
		} else if (m_input.current_token() == TOKEN_NAME_GLOBAL) {
			name = m_input.current_string();
			m_input.next();
			return true;
		} else {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected a name");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
	}


	bool parse_instruction(Instruction &instruction) {
		std::string name;
		SourceRange name_range;
		SourceRange assign_range;

		if (m_input.current_token() == TOKEN_LABEL) {
			LabelInstruction ins;
			ins.name = m_input.current_string();
			m_input.next();
			instruction = ins;
			return true;
		}

		if (m_input.current_token() == TOKEN_NAME_LOCAL) {
			name = m_input.current_string();
			name_range = m_input.current_range();
			m_input.next();

			if (m_input.current_token() == TOKEN_COMMA) {
				CallInstruction ins;
				ins.outputs.push_back(name);
				while (m_input.current_token() == TOKEN_COMMA) {
					m_input.next();
					std::string n;
					if (!parse_name(n))
						return false;
					ins.outputs.push_back(n);
				}

				if (m_input.current_token() != TOKEN_EQUAL) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected '=' after instruction names");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				if (m_input.current_token() != TOKEN_KW_CALL) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected call instruction after multiple return values");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}

				if (!parse_call(ins))
					return false;

				instruction = ins;
				return true;
			}

			if (m_input.current_token() != TOKEN_EQUAL) {
				if (m_dctx) {
					auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected '=' after instruction name '"+name+"'");
					msg->set_main_range(m_input.current_range());
					msg->add_highlit_range(name_range);
					auto d = make_unique<Diagnostic>();
					d->add(std::move(msg));
					m_dctx->add(std::move(d));
				}
				return false;
			}
			assign_range = union_range(name_range, m_input.current_range());
			m_input.next();
		}

		switch (m_input.current_token()) {
			case TOKEN_KW_DRV: {
				m_input.next();
				DriveInstruction ins;

				if (!parse_name(ins.target))
					return false;

				if (m_input.current_token() == TOKEN_KW_CLEAR) {
					m_input.next();
					ins.clear = true;
				} else {
					ins.clear = false;
				}

				if (!parse_value(ins.value))
					return false;

				if (m_input.current_token() == TOKEN_TIME_LITERAL) {
					if (!parse_time(ins.time))
						return false;
					ins.has_time = true;
				}

				instruction = std::move(ins);
				return true;
			} break;

			case TOKEN_KW_ADD: return parse_binary_instruction<AddInstruction>(name, instruction);
			case TOKEN_KW_SUB: return parse_binary_instruction<SubInstruction>(name, instruction);
			case TOKEN_KW_AND: return parse_binary_instruction<AndInstruction>(name, instruction);
			case TOKEN_KW_OR:  return parse_binary_instruction<OrInstruction >(name, instruction);
			case TOKEN_KW_XOR: return parse_binary_instruction<XorInstruction>(name, instruction);

			case TOKEN_KW_RET: {
				m_input.next();
				instruction = RetInstruction();
				return true;
			} break;

			case TOKEN_KW_NOT: {
				m_input.next();
				NotInstruction ins;
				ins.name = name;

				if (!parse_value(ins.value))
					return false;

				instruction = std::move(ins);
				return true;
			} break;

			case TOKEN_KW_WAIT: {
				if (!name.empty()) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "wait instruction cannot have a name");
						msg->set_main_range(m_input.current_range());
						msg->add_highlit_range(assign_range);
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				if (m_input.current_token() == TOKEN_TIME_LITERAL) {
					TimedWaitInstruction ins;
					ins.absolute = false;
					if (!parse_time(ins.time))
						return false;
					instruction = std::move(ins);
					return true;
				} else if (m_input.current_token() == TOKEN_KW_ABS) {
					m_input.next();
					TimedWaitInstruction ins;
					ins.absolute = true;
					if (!parse_time(ins.time))
						return false;
					instruction = std::move(ins);
					return true;
				} else if (m_input.current_token() == TOKEN_KW_COND) {
					m_input.next();
					ConditionalWaitInstruction ins;
					if (!parse_value(ins.cond))
						return false;
					if (!parse_name(ins.dest))
						return false;
					instruction = std::move(ins);
					return true;
				} else {
					instruction = UnconditionalWaitInstruction();
					return true;
				}
			} break;

			case TOKEN_KW_ST: {
				if (!name.empty()) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "store instruction cannot have a name");
						msg->set_main_range(m_input.current_range());
						msg->add_highlit_range(assign_range);
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				StoreInstruction ins;
				if (!parse_value(ins.addr))
					return false;
				if (!parse_value(ins.value))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_LD: {
				m_input.next();

				LoadInstruction ins;
				ins.name = name;

				if (m_input.current_token() != TOKEN_TYPE) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected type");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				ins.type = UnknownType(m_input.current_string());
				m_input.next();

				if (!parse_value(ins.addr))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_BR: {
				if (!name.empty()) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "branch instruction cannot have a name");
						msg->set_main_range(m_input.current_range());
						msg->add_highlit_range(assign_range);
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				Value dest_or_cond;
				if (!parse_value(dest_or_cond))
					return false;

				if (m_input.current_token() == TOKEN_COMMA) {
					m_input.next();

					ConditionalBranchInstruction ins;
					ins.cond = dest_or_cond;

					if (!parse_value(ins.dest_true))
						return false;

					if (m_input.current_token() != TOKEN_COMMA) {
						if (m_dctx) {
							auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected comma followed by negative branch label");
							msg->set_main_range(m_input.current_range());
							auto d = make_unique<Diagnostic>();
							d->add(std::move(msg));
							m_dctx->add(std::move(d));
						}
						return false;
					}
					m_input.next();

					if (!parse_value(ins.dest_false))
						return false;

					instruction = ins;
					return true;
				} else {
					UnconditionalBranchInstruction ins;
					ins.dest = dest_or_cond;
					instruction = ins;
					return true;
				}
			} break;

			case TOKEN_KW_SIG: {
				m_input.next();
				SignalInstruction ins;
				ins.name = name;

				if (!parse_type(ins.type))
					return false;

				if (m_input.current_token() == TOKEN_COMMA) {
					m_input.next();
					if (!parse_value(ins.initial))
						return false;
				}

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_ALLOC: {
				m_input.next();
				AllocInstruction ins;
				ins.name = name;

				if (!parse_type(ins.type))
					return false;

				if (m_input.current_token() == TOKEN_COMMA) {
					m_input.next();
					if (!parse_value(ins.initial))
						return false;
				}

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_CMP: {
				m_input.next();
				CompareInstruction ins;
				ins.name = name;

				switch (m_input.current_token()) {
					case TOKEN_KW_EQ:  ins.type = CMP_TYPE_EQ;  break;
					case TOKEN_KW_NE:  ins.type = CMP_TYPE_NE;  break;
					case TOKEN_KW_SGT: ins.type = CMP_TYPE_SGT; break;
					case TOKEN_KW_SLT: ins.type = CMP_TYPE_SLT; break;
					case TOKEN_KW_SGE: ins.type = CMP_TYPE_SGE; break;
					case TOKEN_KW_SLE: ins.type = CMP_TYPE_SLE; break;
					case TOKEN_KW_UGT: ins.type = CMP_TYPE_UGT; break;
					case TOKEN_KW_ULT: ins.type = CMP_TYPE_ULT; break;
					case TOKEN_KW_UGE: ins.type = CMP_TYPE_UGE; break;
					case TOKEN_KW_ULE: ins.type = CMP_TYPE_ULE; break;
					default: {
						if (m_dctx) {
							auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected comparison type");
							msg->set_main_range(m_input.current_range());
							auto d = make_unique<Diagnostic>();
							d->add(std::move(msg));
							m_dctx->add(std::move(d));
						}
						return false;
					}
				}
				m_input.next();

				if (!parse_value(ins.arga) || !parse_value(ins.argb))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_MUL: {
				m_input.next();
				MulInstruction ins;
				ins.name = name;

				if (!parse_sign(ins.sign))
					return false;

				if (!parse_value(ins.arga) || !parse_value(ins.argb))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_DIV: {
				m_input.next();

				InstructionSign sign;
				if (!parse_sign(sign))
					return false;

				if (m_input.current_token() == TOKEN_KW_MOD) {
					m_input.next();
					ModInstruction ins;
					ins.name = name;
					ins.sign = sign;
					if (!parse_value(ins.arga) || !parse_value(ins.argb))
						return false;
					instruction = ins;
					return true;
				} else if (m_input.current_token() == TOKEN_KW_REM) {
					m_input.next();
					RemInstruction ins;
					ins.name = name;
					ins.sign = sign;
					if (!parse_value(ins.arga) || !parse_value(ins.argb))
						return false;
					instruction = ins;
					return true;
				} else {
					DivInstruction ins;
					ins.name = name;
					ins.sign = sign;
					if (!parse_value(ins.arga) || !parse_value(ins.argb))
						return false;
					instruction = ins;
					return true;
				}
			} break;

			case TOKEN_KW_LMAP: {
				m_input.next();
				LmapInstruction ins;
				ins.name = name;

				if (!parse_type(ins.type))
					return false;
				if (!parse_value(ins.value))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_TRUNC: {
				m_input.next();
				TruncInstruction ins;
				ins.name = name;

				if (!parse_type(ins.type))
					return false;
				if (!parse_value(ins.value))
					return false;

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_EXT: {
				m_input.next();

				if (m_input.current_token() == TOKEN_KW_SIGNED) {
					m_input.next();
					SignExtInstruction ins;
					ins.name = name;

					if (!parse_type(ins.type))
						return false;
					if (!parse_value(ins.value))
						return false;

					instruction = ins;
					return true;
				} else {
					PaddingExtInstruction ins;
					ins.name = name;

					if (!parse_type(ins.type))
						return false;
					if (!parse_value(ins.value) || !parse_value(ins.padding))
						return false;

					instruction = ins;
					return true;
				}
			} break;

			case TOKEN_KW_CAT: {
				m_input.next();
				CatInstruction ins;
				ins.name = name;

				for (;;) {
					std::tuple<Type,Value> t;
					if (!parse_type(std::get<0>(t)) || !parse_value(std::get<1>(t)))
						return false;
					ins.args.push_back(std::move(t));
					if (m_input.current_token() == TOKEN_COMMA)
						m_input.next();
					else
						break;
				}

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_SEL: {
				m_input.next();
				SelInstruction ins;
				ins.name = name;
				if (!parse_type(ins.arg_type) || !parse_value(ins.arg))
					return false;

				while (m_input.current_token() == TOKEN_COMMA) {
					m_input.next();

					std::tuple<unsigned,unsigned> r;
					if (m_input.current_token() != TOKEN_INTEGER_LITERAL) {
						if (m_dctx) {
							auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected an index or a range");
							msg->set_main_range(m_input.current_range());
							auto d = make_unique<Diagnostic>();
							d->add(std::move(msg));
							m_dctx->add(std::move(d));
						}
						return false;
					}
					std::get<0>(r) = std::stoi(m_input.current_string());
					m_input.next();

					if (m_input.current_token() == TOKEN_MINUS) {
						m_input.next();
						if (m_input.current_token() != TOKEN_INTEGER_LITERAL) {
							if (m_dctx) {
								auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected second index of range");
								msg->set_main_range(m_input.current_range());
								auto d = make_unique<Diagnostic>();
								d->add(std::move(msg));
								m_dctx->add(std::move(d));
							}
							return false;
						}
						std::get<1>(r) = std::stoi(m_input.current_string());
						m_input.next();
					} else {
						std::get<1>(r) = std::get<0>(r);
					}

					ins.ranges.push_back(std::move(r));
				}

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_INST: {
				if (!name.empty()) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "inst instruction cannot have a name");
						msg->set_main_range(m_input.current_range());
						msg->add_highlit_range(assign_range);
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				InstInstruction ins;
				if (!parse_name(ins.name))
					return false;

				// input signals
				if (m_input.current_token() != TOKEN_LPAREN) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected instantiation input signals");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				if (m_input.current_token() != TOKEN_RPAREN) {
					if (!parse_inst_args(ins.inputs))
						return false;
				}

				if (m_input.current_token() != TOKEN_RPAREN) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after input signals");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				// output signals
				if (m_input.current_token() != TOKEN_LPAREN) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected instantiation output signals");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				if (m_input.current_token() != TOKEN_RPAREN) {
					if (!parse_inst_args(ins.outputs))
						return false;
				}

				if (m_input.current_token() != TOKEN_RPAREN) {
					if (m_dctx) {
						auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after output signals");
						msg->set_main_range(m_input.current_range());
						auto d = make_unique<Diagnostic>();
						d->add(std::move(msg));
						m_dctx->add(std::move(d));
					}
					return false;
				}
				m_input.next();

				instruction = ins;
				return true;
			} break;

			case TOKEN_KW_CALL: {
				CallInstruction ins;
				if (!name.empty())
					ins.outputs.push_back(name);
				if (!parse_call(ins))
					return false;

				instruction = ins;
				return true;
			} break;

			default: break;
		}

		if (m_dctx) {
			auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected an instruction");
			msg->set_main_range(m_input.current_range());
			auto d = make_unique<Diagnostic>();
			d->add(std::move(msg));
			m_dctx->add(std::move(d));
		}
		return false;
	}


	template <typename T>
	bool parse_binary_instruction(std::string name, Instruction &instruction) {
		m_input.next();
		T ins;
		ins.name = name;
		if (!parse_value(ins.arga) || !parse_value(ins.argb))
			return false;
		instruction = std::move(ins);
		return true;
	}


	/// Parses a value. A value may be a local or global name, or a number
	/// literal. Expects the input lexer to sit on the first token of the value.
	bool parse_value(Value &value) {
		if (m_input.current_token() == TOKEN_NAME_LOCAL || m_input.current_token() == TOKEN_NAME_GLOBAL) {
			value = UnresolvedValue(m_input.current_string());
			m_input.next();
			return true;
		}

		if (m_input.current_token() == TOKEN_NUMBER_LITERAL) {
			value = NumberValue(m_input.current_string());
			m_input.next();
			return true;
		}

		if (m_dctx) {
			auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected a value");
			msg->set_main_range(m_input.current_range());
			auto d = make_unique<Diagnostic>();
			d->add(std::move(msg));
			m_dctx->add(std::move(d));
		}
		return false;
	}


	/// Parses a time interval. Expects the input lexer to sit on the time
	/// literal.
	bool parse_time(Time &time) {
		if (m_input.current_token() != TOKEN_TIME_LITERAL) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected a time interval");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}

		time = Time(m_input.current_string());
		m_input.next();
		return true;
	}


	/// Parses a type. Expects the input lexer to sit on the first token of the
	/// type.
	bool parse_type(Type &type) {
		if (m_input.current_token() != TOKEN_TYPE) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected type");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}

		type = UnknownType(m_input.current_string());
		m_input.next();
		return true;
	}

	/// Parse the sign of an instruction. Expects the input lexer to sit on the
	/// corresponding SIGNED or UNSIGNED keyword token.
	bool parse_sign(InstructionSign &sign) {
		switch (m_input.current_token()) {
			case TOKEN_KW_SIGNED:   sign = INS_SIGNED; break;
			case TOKEN_KW_UNSIGNED: sign = INS_UNSIGNED; break;
			default: {
				if (m_dctx) {
					auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected sign");
					msg->set_main_range(m_input.current_range());
					auto d = make_unique<Diagnostic>();
					d->add(std::move(msg));
					m_dctx->add(std::move(d));
				}
				return false;
			} break;
		}
		m_input.next();
		return true;
	}

	bool parse_inst_args(std::vector<std::string> &args) {
		while (m_input) {
			std::string arg;
			if (!parse_name(arg))
				return false;
			args.push_back(arg);

			if (m_input.current_token() == TOKEN_COMMA)
				m_input.next();
			else
				break;
		}
		return true;
	}

	bool parse_call(CallInstruction &ins) {
		assert(m_input.current_token() == TOKEN_KW_CALL);
		m_input.next();

		if (!parse_name(ins.name))
			return false;

		if (m_input.current_token() != TOKEN_LPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected call arguments");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (!parse_call_args(ins.inputs))
				return false;
		}

		if (m_input.current_token() != TOKEN_RPAREN) {
			if (m_dctx) {
				auto msg = make_unique<DiagnosticMessage>(DIAG_ERROR, "expected closing parenthesis after call arguments");
				msg->set_main_range(m_input.current_range());
				auto d = make_unique<Diagnostic>();
				d->add(std::move(msg));
				m_dctx->add(std::move(d));
			}
			return false;
		}
		m_input.next();

		return true;
	}

	bool parse_call_args(std::vector<Value> &args) {
		while (m_input) {
			Value value;
			if (!parse_value(value))
				return false;
			args.push_back(std::move(value));

			if (m_input.current_token() == TOKEN_COMMA)
				m_input.next();
			else
				break;
		}
		return true;
	}
};


AssemblyReader::AssemblyReader(Assembly &assembly)
	:	m_assembly(assembly) {
}

AssemblyReader& AssemblyReader::operator()(Range<const char*> input, SourceLocation loc, DiagnosticContext *dctx) {
	AssemblyLexer lexer(input, loc, dctx);
	return operator()(lexer, dctx);
}

AssemblyReader& AssemblyReader::operator()(AssemblyLexer &input, DiagnosticContext *dctx) {
	AssemblyReaderInternal rd(input, dctx);
	rd.parse_root(m_assembly);
	return *this;
}

} // namespace llhd
