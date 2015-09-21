/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/diagnostic/diagnostic.hpp"
#include "llhd/diagnostic/source_layout.hpp"
#include "llhd/location.hpp"
#include <map>
#include <string>

namespace llhd {


/// \needsdoc
/// \ingroup diagnostic
class ConsoleDiagnosticPrinter : public DiagnosticConsumer {
	std::function<std::string(SourceId)> path_callback;
	std::function<Range<const char*>(SourceId)> content_callback;

public:
	ConsoleDiagnosticPrinter() {}

	void consume(Diagnostic const& d);

	unsigned line_width = 0;

private:
	std::string lookup_source_path(SourceId id);
	SourceLayout const& lookup_source_layout(SourceId id);

	std::map<SourceId,SourceLayout> m_source_layout_cache;
};


} // namespace llhd
