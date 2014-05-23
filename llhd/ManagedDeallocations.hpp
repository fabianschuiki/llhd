/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <map>

namespace llhd {

struct ManagedDeallocations {
	typedef std::map<void(*)(void*), std::vector<void*> > DeallocationsMap;
	DeallocationsMap deallocations;

public:
	~ManagedDeallocations() {
		for (DeallocationsMap::iterator i = deallocations.begin(); i != deallocations.end(); i++) {
			for (std::vector<void*>::iterator j = i->second.begin(); j != i->second.end(); j++) {
				(i->first)(*j);
			}
		}
	}

	void addDeallocation(void (*callback)(void*), void* data) {
		deallocations[callback].push_back(data);
	}
};

} // namespace llhd
