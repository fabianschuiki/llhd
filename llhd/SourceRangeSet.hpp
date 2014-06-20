/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/range.hpp"
#include "llhd/SourceLocation.hpp"
#include <algorithm>
#include <iostream>
#include <vector>

namespace llhd {

class SourceRangeSet {
	std::vector<SourceRange> ranges;

	struct compare {
		bool operator()(SourceRange a, SourceRange b) { return a.e < b.s; }
	};

public:
	typedef std::vector<SourceRange>::iterator Iterator;
	typedef std::vector<SourceRange>::const_iterator ConstIterator;

	Iterator insert(SourceRange r) {
		Iterator lb = std::lower_bound(
			ranges.begin(), ranges.end(), r, compare());
		Iterator ub = std::upper_bound(
			ranges.begin(), ranges.end(), r, compare());

		// std::cout << "insert " << r << '\n';
		// std::cout << "  into ranges = {";
		// for (auto r : ranges)
		// 	std::cout << ' ' << r;
		// std::cout << " }\n";

		// std::cout << "  lb = " << std::distance(ranges.begin(), lb) << ", ";
		// std::cout << "  ub = " << std::distance(ranges.begin(), ub) << '\n';

		// SourceRange br = r;
		// Modify the range to cover what we are about to replace.
		if (lb != ranges.end()) {
			if (lb->s < r.s)
				r.s = lb->s;
		}
		if (ub != ranges.end()) {
			if (ub != ranges.begin()) {
				auto i = ub-1;
				if (i->e > r.e)
					r.e = i->e;
			}
		} else if (!ranges.empty() && ranges.back().e > r.e) {
			r.e = ranges.back().e;
		}
		// if (br != r)
		// 	std::cout << "  modified " << br << " to " << r << '\n';

		Iterator i = ranges.erase(lb, ub);
		// ranges.push_back(r);


		return ranges.insert(i, r);
	}

	template<class InputIterator>
	void insert(InputIterator first, InputIterator last) {
		while (first != last) {
			insert(*first);
			++first;
		}
	}

	template<class InputIterator>
	void insert(Range<InputIterator> r) {
		insert(r.begin(), r.end());
	}

	unsigned getSize() const { return ranges.size(); }

	ConstIterator begin() const { return ranges.begin(); }
	ConstIterator end() const { return ranges.end(); }
};

} // namespace llhd
