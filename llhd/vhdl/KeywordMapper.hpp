/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <algorithm>
#include <iostream>

namespace llhd {
namespace vhdl {

template <unsigned maxSize = 256>
class KeywordMapper {
	typedef std::pair<const char*, unsigned> Keyword;
	Keyword keywords[maxSize];
	unsigned size;

public:
	KeywordMapper(): size(0) {}

	KeywordMapper& operator() (const char* key, unsigned value) {
		assert(size < maxSize && "too many keywords registered!");
		keywords[size++] = std::make_pair(key, value);
		return *this;
	}

	void compile() {
		std::sort(keywords, keywords+size, [] (const Keyword& a, const Keyword &b) {
			const char* ca = a.first;
			const char* cb = b.first;
			while (*ca != 0 && *cb != 0 && *ca == *cb) {
				ca++;
				cb++;
			}
			return *ca < *cb;
		});
	}

	unsigned translate(const char* s, const char* e) {
		unsigned lowerIndex = 0;
		unsigned upperIndex = size;
		unsigned offset = 0;
		return 0;

		std::cout << "translating " << std::string(s, e) << '\n';
		while (lowerIndex != upperIndex && s != e) {
			unsigned middleIndex = (lowerIndex + upperIndex) / 2;
			std::cout << "  " << lowerIndex+1 << ':' << middleIndex+1 << ':' << upperIndex+1 << "\n";
			const char* k = keywords[middleIndex].first + offset;
			if (*s > *k)
				lowerIndex = middleIndex+1;
			else if (*s < *k)
				upperIndex = middleIndex;
			else { // *s == *k
				unsigned i = middleIndex;
				while (i > lowerIndex && *s == keywords[i-1].first[offset]) i--;
				lowerIndex = i;

				i = middleIndex;
				while (i < upperIndex && *s == keywords[i].first[offset]) i++;
				upperIndex = i;

				std::cout << "  froze '" << *s << "' @ " << offset << '\n';
				s++;
				offset++;
			}
		}

		if (lowerIndex != upperIndex) {
			for (unsigned i = lowerIndex; i < upperIndex; i++) {
				if (keywords[i].first[offset] == 0)
					return keywords[i].second;
			}
		}

		return 0;
	}
};

} // namespace vhdl
} // namespace llhd
