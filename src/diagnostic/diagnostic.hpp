/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/location.hpp"
#include "llhd/utils/iterator.hpp"
#include "llhd/utils/range.hpp"
#include <memory>
#include <functional>
#include <set>
#include <vector>


namespace llhd {

class Diagnostic;
class DiagnosticConsumer;
class DiagnosticContext;
class DiagnosticMessage;


/// \needsdoc
/// \ingroup diagnostic
enum DiagnosticSeverity {
	DIAG_FATAL,
	DIAG_ERROR,
	DIAG_WARNING,
	DIAG_INFO,
	DIAG_NONE
};


/// \needsdoc
/// \ingroup diagnostic
class DiagnosticConsumer {
public:
	virtual void consume(Diagnostic const& d) = 0;
};


/// \needsdoc
/// \ingroup diagnostic
class DiagnosticContext {
	typedef std::vector<std::unique_ptr<Diagnostic>> diagnostics_type;
	DiagnosticSeverity severity = DIAG_NONE;
	diagnostics_type diagnostics;

public:
	typedef DereferencingIterator<diagnostics_type::iterator> iterator;
	typedef DereferencingIterator<diagnostics_type::const_iterator> const_iterator;

	void add(std::unique_ptr<Diagnostic>&& d);

	DiagnosticSeverity get_severity() const { return severity; }

	bool is_fatal()   const { return severity <= DIAG_FATAL; }
	bool is_error()   const { return severity <= DIAG_ERROR; }
	bool is_warning() const { return severity <= DIAG_WARNING; }
	bool is_info()    const { return severity <= DIAG_INFO; }

	void each_diagnostic(std::function<void(Diagnostic&)> fn) {
		for (auto const& d : diagnostics)
			fn(*d);
	}

	void eachDiagnostic(std::function<void(Diagnostic const&)> fn) const {
		for (auto const& d : diagnostics)
			fn(*d);
	}

	Range<iterator> get_diagnostics() {
		return make_range(
			iterator(diagnostics.begin()), iterator(diagnostics.end())
		);
	}

	Range<const_iterator> get_diagnostics() const {
		return make_range(
			const_iterator(diagnostics.begin()), const_iterator(diagnostics.end())
		);
	}
};


/// \needsdoc
/// \ingroup diagnostic
class Diagnostic {
	typedef std::vector<std::unique_ptr<DiagnosticMessage>> messages_type;
	friend class DiagnosticContext;

	unsigned id;
	DiagnosticSeverity severity = DIAG_NONE;
	messages_type messages;

public:
	typedef DereferencingIterator<messages_type::iterator> iterator;
	typedef DereferencingIterator<messages_type::const_iterator> const_iterator;

	Diagnostic(unsigned id = 0): id(id) {}

	void add(std::unique_ptr<DiagnosticMessage>&& msg);

	unsigned get_id() const { return id; }
	DiagnosticSeverity get_severity() const { return severity; }

	void each_message(std::function<void(DiagnosticMessage&)> fn) {
		for (auto const& msg : messages)
			fn(*msg);
	}

	void each_message(std::function<void(DiagnosticMessage const&)> fn) const {
		for (auto const& msg : messages)
			fn(*msg);
	}

	Range<iterator> get_messages() {
		return make_range(
			iterator(messages.begin()), iterator(messages.end())
		);
	}

	Range<const_iterator> get_messages() const {
		return make_range(
			const_iterator(messages.begin()), const_iterator(messages.end())
		);
	}
};


/// \needsdoc
/// \ingroup diagnostic
class DiagnosticMessage {
public:
	DiagnosticMessage(DiagnosticSeverity severity, std::string const& text):
		severity(severity),
		text(text) {}

	DiagnosticSeverity get_severity() const { return severity; }
	std::string const& get_text() const { return text; }
	SourceRange const& get_main_range() const { return main_range; }
	std::vector<SourceRange> const& get_highlit_ranges() const { return highlit_ranges; }
	std::vector<SourceRange> const& get_visible_ranges() const { return visible_ranges; }

	void set_main_range(SourceRange const& r) { main_range = r; }
	void add_highlit_range(SourceRange const& r) { highlit_ranges.push_back(r); }
	void add_visible_range(SourceRange const& r) { visible_ranges.push_back(r); }

private:
	friend class Diagnostic;

	DiagnosticSeverity severity;
	std::string text;
	SourceRange main_range;
	std::vector<SourceRange> highlit_ranges;
	std::vector<SourceRange> visible_ranges;
};


} // namespace llhd
