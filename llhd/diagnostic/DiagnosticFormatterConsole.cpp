/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/console.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/SourceRangeSet.hpp"
#include "llhd/diagnostic/Diagnostic.hpp"
#include "llhd/diagnostic/DiagnosticFormatterConsole.hpp"
#include "llhd/diagnostic/DiagnosticMessage.hpp"
#include <functional>
#include <iterator>
#include <sstream>
#include <cstring>
using namespace llhd;


template<class InputIterator>
InputIterator findEnclosingRange(
    InputIterator first,
    InputIterator last,
    SourceRange r) {

    while (first != last) {
        if (first->s <= r.s && first->e >= r.e)
            break;
        ++first;
    }
    return first;
}

class FormattingIterator {
    /// Pointer to the current position in the string to be formatted.
    const char* fmt;
    /// Function formats the individual arguments.
    std::function<std::string(unsigned)> fmtarg;
    /// Currently active substitution.
    std::string sub;
    /// Iterator into the currently active substitution.
    std::string::iterator subit;

    inline void substitute() {
        if (*fmt == '$') {
            ++fmt;
            assert(*fmt >= '0' && *fmt <= '9');
            sub = fmtarg(*fmt-'0');
            subit = sub.begin();
        } else if (*fmt == 0) {
            fmt = nullptr;
        }
    }

public:
    FormattingIterator(
        const char* fmt,
        std::function<std::string(unsigned)> fmtarg):

        fmt(fmt),
        fmtarg(fmtarg) {
        substitute();
    }

    FormattingIterator(): fmt(nullptr), fmtarg() {}

    FormattingIterator& operator++() {
        if (!sub.empty()) {
            ++subit;
            if (subit == sub.end()) {
                sub.clear();
            } else {
                return *this;
            }
        }
        ++fmt;
        substitute();
        return *this;
    }

    FormattingIterator operator++(int) {
        FormattingIterator i(*this);
        operator++();
        return i;
    }

    char operator*() const {
        return !sub.empty() ? *subit : *fmt;
    }

    bool operator==(const FormattingIterator& rhs) const {
        return fmt == rhs.fmt;
    }
    bool operator!=(const FormattingIterator& rhs) const {
        return fmt != rhs.fmt;
    }
};


static const char* getLabelForType(DiagnosticType type) {
    switch (type) {
        case kFatal:   return "fatal error";
        case kError:   return "error";
        case kWarning: return "warning";
        case kNote:    return "note";
        case kFixit:   return "fixit";
    }
    return "unspecified";
}

static const char* beginLabelForType(DiagnosticType type) {
    switch (type) {
        case kFatal:   return "\033[31;1m";
        case kError:   return "\033[31;1m";
        case kWarning: return "\033[33;1m";
        case kNote:    return "\033[1m";
        case kFixit:   return "\033[1m";
    }
    return "";
}

static const char* endLabel() {
    return "\033[0m";
}

template<class InputIterator, class OutputIterator>
void indent(
    InputIterator first,
    InputIterator last,
    OutputIterator& out,
    unsigned indentation) {

    while (first != last) {
        *out++ = *first;
        if (*first == '\n') {
            for (unsigned i = 0; i < indentation; i++)
                *out++ = ' ';
        }
        ++first;
    }
}

template<class InputIterator>
class LinebreakingIterator {
    InputIterator first, last;
    unsigned width;
    unsigned pos;
    bool inTag, insertBreak;

    void init() {
        pos = 0;
        inTag = false;
        insertBreak = false;
    }

public:
    LinebreakingIterator(
        InputIterator first,
        InputIterator last,
        unsigned w):

        first(first),
        last(last),
        width(w) {

        init();
    }

    LinebreakingIterator() { init(); }

    LinebreakingIterator& operator++() {
        if (insertBreak) {
            insertBreak = false;
            pos = 0;
            return *this;
        }

        if (*first == '\033') {
            inTag = true;
            ++first;
        } else if (inTag) {
            if (*first == 'm')
                inTag = false;
            ++first;
        } else if (*first == '\n') {
            pos = 0;
            ++first;
        } else {
            if (width > 0 && pos+1 == width) {
                insertBreak = true;
                pos = 0;
                ++first;
                if (*first == '\n' || *first == ' ')
                    ++first;
            } else {
                ++first;
                ++pos;
            }
        }

        return *this;
    }

    LinebreakingIterator operator++(int) {
        LinebreakingIterator tmp(*this);
        operator++();
        return tmp;
    }

    char operator*() const {
        if (insertBreak)
            return '\n';
        return *first;
    }

    bool operator==(const LinebreakingIterator& rhs) const {
        return first == rhs.first;
    }
    bool operator!=(const LinebreakingIterator& rhs) const {
        return first != rhs.first;
    }
};

/// Highlights a portion of the input range. In the range [\a first, \a last),
/// which is assumed to coincide with \a rng, replaces all characters in the
/// range \a hl with the character \a hc.
///
/// @return Returns true if any replacement happened, false otherwise.
template<class InputIterator>
bool highlight(
    InputIterator first,
    InputIterator last,
    SourceRange rng,
    SourceRange hl,
    char hc) {

    if (rng.s <= hl.s && rng.e > hl.s) {
        unsigned start = hl.s - rng.s;
        unsigned end = hl.e - rng.s;

        InputIterator istart = first+start;
        InputIterator iend = first+end;
        while (istart != last && istart != iend)
            *istart++ = hc;

        return true;
    }
    return false;
}


DiagnosticFormatterConsole& DiagnosticFormatterConsole::operator<<(
    const Diagnostic* diag) {

    // In case we are formatting into stdout, try to lookup the width of the
    // console window so we may break the lines of the output in a nice way.
    unsigned outputWidth = 0;
    if (breakLinesToTerminalSize) {
        outputWidth = getTerminalWidth();
    }

    std::vector<SourceRange> printedRanges;

    for (unsigned i = 0; i < diag->getNumMessages(); i++) {
        const DiagnosticMessage* msg = diag->getMessage(i);

        // Output the label for this message, potentially colored, and calculate
        // the indentation for the message.
        unsigned indentation = 0;
        if (i != 0) {
            output << "  ";
            indentation += 2;
        }

        const char* label = getLabelForType(msg->getType());
        indentation += strlen(label) + 2;
        output << beginLabelForType(msg->getType())
               << label << ':'
               << endLabel() << ' ';

        // Calculate the position at which a line break needs to be introduced
        // given the current outputWidth.
        unsigned indentedWidth = 0;
        if (outputWidth > 0)
            indentedWidth = outputWidth - indentation;

        // Calculate the source code snippets to show in the console.
        SourceRangeSet rngs;
        if (msg->getMainRange().isValid()) {
            rngs.insert(msg->getMainRange());
        }
        rngs.insert(msg->getHighlightedRanges());
        rngs.insert(msg->getRelevantRanges());

        // Also include the source locations set as arguments of the message.
        // If one of the ranges listed as arguments is already fully covered by
        // one of the ranges previously printed there is no need to add it
        // again.
        for (auto arg : msg->getArguments()) {
            if (arg.type == DiagnosticMessageArgument::kSourceRange) {
                auto i = findEnclosingRange(
                    printedRanges.begin(),
                    printedRanges.end(),
                    arg.r);
                if (i == printedRanges.end())
                    rngs.insert(arg.r);
            }
        }

        // Add the ranges of the set to the list of printed ranges such that the
        // message may refer to them by index.
        unsigned rngsOffset = printedRanges.size();
        printedRanges.insert(printedRanges.end(), rngs.begin(), rngs.end());

        // Print the formatted message. To this end we first create a
        // FormattingIterator that uses a lambda expression to obtain and format
        // the individual arguments.
        FormattingIterator fmtbegin(
            msg->getMessage(),
            [&printedRanges,msg,this](unsigned idx){
                const DiagnosticMessageArgument& arg = msg->getArgument(idx);
                std::stringstream s;
                switch (arg.type) {
                    case DiagnosticMessageArgument::kSignedInt:
                        s << arg.i; break;
                    case DiagnosticMessageArgument::kUnsignedInt:
                        s << arg.u; break;
                    case DiagnosticMessageArgument::kString:
                        s << arg.s; break;
                    case DiagnosticMessageArgument::kSourceRange: {
                        auto i = findEnclosingRange(
                            printedRanges.begin(),
                            printedRanges.end(),
                            arg.r);
                        if (i == printedRanges.end()) {
                            PresumedRange rng = manager.getPresumedRange(arg.r);
                            s << "(" << manager.getBufferName(rng.s.fid)
                                   << ':' << rng << ')';
                        } else {
                            auto id = std::distance(printedRanges.begin(), i)+1;
                            s << '(' << id << ')';
                        }
                    } break;
                    default:
                        s << "<unknown arg " << idx << '>'; break;
                }
                return s.str();
            });
        FormattingIterator fmtend;

        LinebreakingIterator<FormattingIterator>
            lbbegin(fmtbegin, fmtend, indentedWidth),
            lbend;

        std::ostreambuf_iterator<char> oit(output);
        indent(lbbegin, lbend, oit, indentation);
        output << '\n';

        // Print the source code snippets.
        for (SourceRangeSet::ConstIterator i = rngs.begin();
            i != rngs.end(); ++i) {

            output << '\n';

            SourceRange sr = *i;
            PresumedRange pr = manager.getPresumedRange(sr);

            // Print the index and buffer name (e.g. "(1) system.vhd:1-3:").
            unsigned idx = std::distance(rngs.begin(), i)+1+rngsOffset;
            output << "  \033[1m(" << idx << ") ";
            output << manager.getBufferName(pr.s.fid) << ':' << pr.s.line;
            if (pr.s.line != pr.e.line)
                output << '-' << pr.e.line;
            output << ":\033[0m\n";

            // Fetch the source code from the manager and search for the start
            // of the first and the end of the last line.
            SourceBuffer src = manager.getBuffer(pr.s.fid);
            const utf8char* p = src.getStart() + pr.s.offset - pr.s.column + 1;
            const utf8char* end = src.getStart() + pr.e.offset;
            while (end != src.getEnd() && *end != '\n')
                end++;

            // Print each source line individually, properly indenting after \n
            // and adding the annotation lines.
            SourceRange lr;
            lr.s = sr.s;
            lr.e = sr.s;
            const char* lead = "   |  ";
            output << lead << "\033[37m";
            while (true) {
                if (p == end || *p == '\n') {
                    output << "\033[0m";

                    // Create an empty annotation line. Then highlight the
                    // ranges requested by the message.
                    std::string annotations(lr.getLength(), ' ');
                    bool anyAnnotations = false;

                    for (auto r : msg->getHighlightedRanges()) {
                        if (highlight(
                                annotations.begin(),
                                annotations.end(),
                                lr,
                                r,
                                '~')) {
                            anyAnnotations = true;
                        }
                    }

                    if (msg->getMainRange().isValid()) {
                        if (highlight(
                                annotations.begin(),
                                annotations.end(),
                                lr,
                                msg->getMainRange(),
                                '^')) {
                            anyAnnotations = true;
                        }
                    }

                    // If anything was highlighted in the annotation line, print
                    // it. Otherwise print nothing to avoid empty lines.
                    if (anyAnnotations) {
                        output << '\n' << lead;
                        output << "\033[36m" << annotations << "\033[0m";
                    }

                    // Break out of the loop if we have reached the end.
                    if (p == end)
                        break;
                    output << '\n' << lead << "\033[37m";

                    // Advance the line range lr to the beginning of the next
                    // line.
                    lr.s = lr.e+1;
                    lr.e = lr.s;
                } else if (*p == '\t') {
                    output << "    ";
                    lr.e += 4;
                } else if (*p != '\r') {
                    output << *p;
                    lr.e += 1;
                }
                p++;
            }
            output << '\n';
        }

        // Insert a newline after the last source snippet.
        output << '\n';
    }

    return *this;
}
