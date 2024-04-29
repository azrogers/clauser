#pragma once

#include <cstdint>
#include <map>
#include <vector>
#include <string>
#include <string_view>
#include <list>

#include "ClTypes.h"

class ClParseState {
public:
    ClParseState(const std::string& inputText) :
        inputText(inputText)
    {
    }

    ClStringTableId AddString(const std::string_view& string);
    ClStringTableId AddString(const std::string& string);

    const std::string_view& LookupString(const ClStringTableId& id) const;
    const std::string& GetInputText() const { return this->inputText; }

private:
    std::string inputText;

    std::vector<std::string_view> stringTable;
    std::list<std::string> storedStrings;
    std::map<uint64_t, ClStringTableId> stringTableLookup;
};