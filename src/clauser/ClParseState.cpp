#include "ClParseState.h"

#include <xxhash.h>
#include <spdlog/spdlog.h>

const uint64_t seed = 0x151bb6d5a03b4a00;

ClStringTableId ClParseState::AddString(const std::string_view& string)
{
	const XXH64_hash_t hash = XXH64(string.data(), string.length(), seed);

	const auto it = this->stringTableLookup.find(hash);
	if (it != this->stringTableLookup.end()) {
		// already present in the string table, return index
		return it->second;
	}

	const ClStringTableId newIndex = this->stringTable.size();
	this->stringTable.push_back(string);

	this->stringTableLookup.emplace(hash, newIndex);

	return newIndex;
}

ClStringTableId ClParseState::AddString(const std::string& string) { 
	const XXH64_hash_t hash = XXH64(string.data(), string.length(), seed);

	const auto it = this->stringTableLookup.find(hash);
	if (it != this->stringTableLookup.end()) {
		// already present in the string table, return index
		return it->second;
	}

	const ClStringTableId newIndex = this->stringTable.size();
	this->storedStrings.push_back(string);
	std::string& storedStringRef = this->storedStrings.back();
	this->stringTable.push_back(storedStringRef);
	this->stringTableLookup.emplace(hash, newIndex);

	return newIndex;
}

const std::string_view& ClParseState::LookupString(const ClStringTableId& id) const
{
	if (id < 0 || id >= this->stringTable.size()) {
		spdlog::error("invalid string table id {}", id);
		return std::string_view();
	}

	return this->stringTable.at(id);
}
