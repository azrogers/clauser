#include "ClClassMapping.h"
#include "ClClassMapping.h"
#include "ClClassMapping.h"
#include "ClClassMapping.h"
#include "ClClassMapping.h"

#include <string_view>
#include <spdlog/spdlog.h>

#include "ClUtil.h"

bool ClClassMapping::ReadObject(ClReader& reader, ClParseState& state, ClParseStatus& outStatus)
{
    return this->ReadObjectProperties(reader, state, outStatus);
}

bool ClClassMapping::ReadValue(ClReader& reader,
                               ClParseState& state,
                               ClValueMapping<void>* mapping,
                               ClRealType realType,
                               ClValueTypes types,
                               void* target,
                               ClParseStatus& outStatus)
{
    switch (realType) {
    case ClRealType::Number:
        return this->ReadNumberValue(reader, state, types, target, outStatus);
    case ClRealType::String:
        return this->ReadStringValue(reader, state, types, target, outStatus);
    case ClRealType::Identifier:
        return this->ReadIdentifierValue(reader, state, types, target, outStatus);
    case ClRealType::Boolean:
        return this->ReadBooleanValue(reader, state, types, target, outStatus);
    case ClRealType::ObjectOrArray:
        if (mapping == nullptr) {
            outStatus.SetError(
                ClParseStatusType::Unsupported,
                ClParseError(
                    "attempted to read object or array without mapping - is this an array within an array?",
                    reader.GetPosition()));
            return false;
        }

        if (types.HasType(ClValueType::Object)) {
            return this->ReadObjectValue(reader, state, *mapping, types, target, outStatus);
        } else if (types.HasType(ClValueType::Array)) {
            return this->ReadArrayValue(reader, state, *mapping, types, target, outStatus);
        }
    }

    outStatus.SetError(
        ClParseStatusType::Unsupported,
        ClParseError(fmt::format("can't read value of type {}", realType), reader.GetPosition()));
    return false;
}

bool ClClassMapping::ReadObjectProperties(ClReader& reader, ClParseState& state, ClParseStatus& outStatus)
{
    ClStringTableId propertyKey;
    ClRealType propertyType;
    while (reader.NextProperty(state, propertyKey, propertyType, outStatus)) {
        // some sort of error we haven't caught, bail
        if (!outStatus.IsOk()) {
            return false;
        }

        auto it = this->mappingTable.find(propertyKey);
        if (it == this->mappingTable.end()) {
            outStatus.SetError(
                ClParseStatusType::UnknownKeyError,
                ClParseError(fmt::format("found unknown identifier '{}', don't know how to handle",
                                         state.LookupString(propertyKey)),
                             reader.GetPosition()));
            return false;
        }

        ClValueMapping<void>& mappingRef = it->second;
        if (!mappingRef.types.HasRealType(propertyType)) {
            // property isn't the right type
            outStatus.SetError(ClParseStatusType::TypeMismatchError,
                               ClParseError(fmt::format("parsed type {} is invalid for property {}",
                                                        propertyType,
                                                        state.LookupString(propertyKey)),
                                            reader.GetPosition()));
            return false;
        }

        // actually read and apply the property to the object
        if (!this->ReadValue(reader,
                             state,
                             &mappingRef,
                             propertyType,
                             mappingRef.types,
                             mappingRef.targetValue,
                             outStatus)) {
            return false;
        }
    }

    return outStatus.IsOk();
}

bool ClClassMapping::ReadNumberValue(
    ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus)
{
    if (types.HasType(ClValueType::Integer)) {
        return reader.ReadInteger(state, *reinterpret_cast<ClInteger*>(target), outStatus);
    } else if (types.HasType(ClValueType::Integer64)) {
        return reader.ReadInteger64(state, *reinterpret_cast<ClInteger64*>(target), outStatus);
    } else if (types.HasType(ClValueType::Decimal)) {
        return reader.ReadDecimal(state, *reinterpret_cast<ClDecimal*>(target), outStatus);
    } else if (types.HasType(ClValueType::Decimal64)) {
        return reader.ReadDecimal64(state, *reinterpret_cast<ClDecimal64*>(target), outStatus);
    }

    outStatus.SetError(ClParseStatusType::TypeMismatchError,
                       ClParseError("no valid number types for value", reader.GetPosition()));
    return false;
}

bool ClClassMapping::ReadStringValue(
    ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus)
{
    ClStringTableId stringId;
    if (!reader.ReadString(state, stringId, outStatus)) {
        return false;
    }

    std::string* destPtr = reinterpret_cast<std::string*>(target);
    std::string_view segment = state.LookupString(stringId);
    *destPtr = std::string(segment.data(), segment.length());

    outStatus.SetStatusType(ClParseStatusType::Ok);

    return true;
}

bool ClClassMapping::ReadIdentifierValue(
    ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus)
{
    ClStringTableId stringId;
    if (!reader.ReadIdentifier(state, stringId, outStatus)) {
        return false;
    }

    std::string* destPtr = reinterpret_cast<std::string*>(target);
    std::string_view segment = state.LookupString(stringId);
    *destPtr = std::string(segment.data(), segment.length());

    outStatus.SetStatusType(ClParseStatusType::Ok);

    return true;
}

bool ClClassMapping::ReadBooleanValue(
    ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus)
{
    return reader.ReadBoolean(state, *reinterpret_cast<ClBoolean*>(target), outStatus);
}

#define MAKE_MAPPING_CASE_EX(TypeName, EnumName, RealType)                                                   \
    else if (mapping.innerTypes.HasType(ClValueType::##EnumName))                                            \
    {                                                                                                        \
        if (!this->ReadArrayValues<TypeName>(reader, state, mapping, RealType, target, outStatus)) {         \
            return false;                                                                                    \
        }                                                                                                    \
    }

#define MAKE_MAPPING_CASE(TypeName, RealType) MAKE_MAPPING_CASE_EX(Cl##TypeName, TypeName, RealType)

bool ClClassMapping::ReadArrayValue(ClReader& reader,
                                    ClParseState& state,
                                    ClValueMapping<void>& mapping,
                                    ClValueTypes types,
                                    void* target,
                                    ClParseStatus& outStatus)
{
    if (!reader.BeginReadArray(state, outStatus)) {
        return false;
    }
    MAKE_MAPPING_CASE(Integer, ClRealType::Number)
    MAKE_MAPPING_CASE(Integer64, ClRealType::Number)
    MAKE_MAPPING_CASE(Decimal, ClRealType::Number)
    MAKE_MAPPING_CASE(Decimal64, ClRealType::Number)
    MAKE_MAPPING_CASE(Boolean, ClRealType::Boolean)
    MAKE_MAPPING_CASE_EX(std::string, String, ClRealType::String)
    MAKE_MAPPING_CASE_EX(std::string, Identifier, ClRealType::Identifier)
    else
    {
        outStatus.SetError(ClParseStatusType::Unsupported,
                           ClParseError("unsupported value type for array", reader.GetPosition()));
        return false;
    }

    if (!outStatus.IsOk()) {
        return false;
    }

    return reader.EndReadArray(state, outStatus);
}

bool ClClassMapping::ReadObjectValue(ClReader& reader,
                                     ClParseState& state,
                                     ClValueMapping<void>& mapping,
                                     ClValueTypes types,
                                     void* target,
                                     ClParseStatus& outStatus)
{
    if (!reader.BeginReadObject(state, outStatus)) {
        return false;
    }

    if (!mapping.innerMapping) {
        outStatus.SetError(ClParseStatusType::InvalidState,
                           ClParseError("missing innerMapping for object value", reader.GetPosition()));
        return false;
    }

    if (!mapping.innerMapping->ReadObjectProperties(reader, state, outStatus)) {
        return false;
    }

    return reader.EndReadObject(state, outStatus);
}

template class ClValueMapping<void>;
template class ClValueMapping<ClInteger>;
template class ClValueMapping<ClInteger64>;
template class ClValueMapping<ClDecimal>;
template class ClValueMapping<ClDecimal64>;
template class ClValueMapping<ClBoolean>;
template class ClValueMapping<std::string>;

template class ValueTypesForTemplateType<void>;
template class ValueTypesForTemplateType<std::string>;
template class ValueTypesForTemplateType<ClInteger>;
template class ValueTypesForTemplateType<ClInteger64>;
template class ValueTypesForTemplateType<ClDecimal>;
template class ValueTypesForTemplateType<ClDecimal64>;
template class ValueTypesForTemplateType<ClBoolean>;