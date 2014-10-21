/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

namespace llhd {

class SimulationDependency {
	bool outdated;

public:
	SimulationDependency(): outdated(false) {}

	bool markOutdated() {
		if (outdated)
			return false;
		outdated = true;
		return true;
	}

	void clearOutdated() { outdated = false; }
	bool isOutdated() const { return outdated; }
};

} // namespace llhd
