/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/diagnostic/Diagnostic.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticMessage.hpp"

namespace llhd {

class DiagnosticBuilder {
	DiagnosticContext& ctx;
	Diagnostic* diag;
	DiagnosticMessage* msg;

	DiagnosticBuilder(const DiagnosticBuilder&);

public:
	template<typename... Args>
	DiagnosticBuilder(DiagnosticContext& ctx, Args... args): ctx(ctx) {
		diag = ctx.alloc.one<Diagnostic>();
		msg = ctx.alloc.one<DiagnosticMessage>(args...);
	}

	DiagnosticBuilder(DiagnosticBuilder&& old): ctx(old.ctx) {
		diag = old.diag;
		msg = old.msg;
		old.diag = nullptr;
		old.msg = nullptr;
	}

	~DiagnosticBuilder() {
		end();
	}

	template<typename... Args>
	DiagnosticBuilder& message(Args... args) {
		if (msg) {
			diag->addMessage(msg);
		}
		msg = ctx.alloc.one<DiagnosticMessage>(args...);
		return *this;
	}

	void end() {
		if (msg) {
			diag->addMessage(msg);
			msg = nullptr;
		}
		if (diag) {
			ctx.addDiagnostic(diag);
			diag = nullptr;
		}
	}

	template<typename T>
	DiagnosticBuilder& arg(const T& a) {
		assert(msg);
		msg->addArgument(a);
		return *this;
	}

	template<typename T>
	DiagnosticBuilder& arg(const T* first, const T* last) {
		unsigned size = (last-first)*sizeof(T);
		char* s = (char*)ctx.alloc.allocate(size + 1);
		std::copy(first, last, s);
		s[size] = 0;
		msg->addArgument(s);
		return *this;
	}

	DiagnosticBuilder& highlight(const SourceRange& rng) {
		assert(msg);
		msg->addHighlightedRange(rng);
		return *this;
	}

	DiagnosticBuilder& main(const SourceRange& rng) {
		assert(msg);
		assert(!msg->getMainRange().isValid());
		msg->setMainRange(rng);
		return *this;
	}
};

} // namespace llhd
