#pragma once

#include <map>
#include <initializer_list>
#include <vector>

#include "ClTypes.h"
#include "ClParseState.h"
#include "ClReader.h"

/**
 * A bitmask of \ref ClValueType
 */
struct ClValueTypes {
	constexpr ClValueTypes() :
		bitmask(0)
	{
	}

	constexpr ClValueTypes(std::initializer_list<ClValueType> valueTypes)
	{
		for (ClValueType type : valueTypes) {
			this->AddType(type);
		}
	}

	constexpr ClValueTypes(ClValueType typeA, ClValueType typeB) :
		bitmask(this->TypeToMask(typeA) | this->TypeToMask(typeB))
	{
	}

	constexpr ClValueTypes(ClValueType type) :
		bitmask(this->TypeToMask(type))
	{
	}

	ClValueTypes(const ClValueTypes& other) :
		bitmask(other.bitmask)
	{
	}

	constexpr void AddType(ClValueType type) { this->bitmask |= this->TypeToMask(type); }
	constexpr void RemoveType(ClValueType type) { this->bitmask = this->bitmask & ~this->TypeToMask(type); }
	constexpr bool HasType(ClValueType type)
	{
		const uint32_t mask = this->TypeToMask(type);
		return (this->bitmask & mask) == mask;
	}
	constexpr bool HasRealType(ClRealType realType)
	{
		switch (realType) {
		case ClRealType::ObjectOrArray:
			return HasType(ClValueType::Object) || HasType(ClValueType::Array);
		case ClRealType::Number:
			return HasType(ClValueType::Integer) || HasType(ClValueType::Integer64)
				|| HasType(ClValueType::Decimal) || HasType(ClValueType::Decimal64);
		case ClRealType::String:
			return HasType(ClValueType::String);
		case ClRealType::Identifier:
			return HasType(ClValueType::Identifier);
		case ClRealType::Boolean:
			return HasType(ClValueType::Boolean);
		}
	}

private:
	constexpr uint32_t TypeToMask(ClValueType type) { return 1 << static_cast<uint32_t>(type); }

	uint32_t bitmask;
};

template <typename T> struct ValueTypesForTemplateType;

template <> struct ValueTypesForTemplateType<ClInteger> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Integer);
};

template <> struct ValueTypesForTemplateType<ClInteger64> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Integer64);
};

template <> struct ValueTypesForTemplateType<ClDecimal> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Decimal);
};

template <> struct ValueTypesForTemplateType<ClDecimal64> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Decimal64);
};

template <> struct ValueTypesForTemplateType<ClBoolean> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Boolean);
};

template <> struct ValueTypesForTemplateType<std::string> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::String, ClValueType::Identifier);
};

template <> struct ValueTypesForTemplateType<void> {
	static constexpr ClValueTypes types = ClValueTypes(ClValueType::Invalid);
};

class ClClassMapping;

template <typename T> struct ClValueMapping {
	static ClValueMapping<T> Create(ClValueTypes types, T* target)
	{
		return ClValueMapping<T>(types, target);
	}

	static ClValueMapping<T> Create(ClClassMapping& objectMapping, T* target)
	{
		return ClValueMapping<T>(ClValueTypes(ClValueType::Object), target, &objectMapping);
	}

	static ClValueMapping<std::vector<T>> Create(std::vector<T>* target)
	{
		return ClValueMapping<std::vector<T>>(
			ClValueTypes(ClValueType::Array), ValueTypesForTemplateType<T>::types, target);
	}

	// a mask representing the types that this mapping fits
	ClValueTypes types;
	// inner types used for arrays
	ClValueTypes innerTypes;
	// inner mapping used for objects
	ClClassMapping* innerMapping = nullptr;
	T* targetValue;

	ClValueMapping(T* target) :
		types(ValueTypesForTemplateType<T>::types),
		innerTypes(),
		targetValue(target)
	{
	}

	template <typename OtherT>
	ClValueMapping(const ClValueMapping<OtherT>& other) :
		types(other.types),
		innerTypes(other.innerTypes),
		innerMapping(other.innerMapping),
		targetValue(static_cast<T*>(other.targetValue))
	{
	}

	ClValueMapping(ClValueTypes types, T* targetValue) :
		types(types),
		targetValue(targetValue)
	{
	}

	ClValueMapping(ClValueTypes types, ClValueTypes innerTypes, T* targetValue) :
		types(types),
		innerTypes(innerTypes),
		targetValue(targetValue)
	{
	}

	ClValueMapping(ClValueTypes types, T* targetValue, ClClassMapping* innerMapping) :
		types(types),
		targetValue(targetValue),
		innerMapping(innerMapping)
	{
	}
};

class ClClassMapping {
public:
	ClClassMapping(ClParseState& state) :
		parseState(state)
	{
	}

	template <typename T> void AddMapping(const std::string& key, const ClValueMapping<T>& mapping)
	{
		const ClStringTableId keyId = this->parseState.AddString(key);
		this->mappingTable.emplace(keyId, static_cast<ClValueMapping<void>>(mapping));
	}

	template <typename T> void AddMapping(const std::string& key, T* target)
	{
		this->AddMapping<T>(key, ClValueMapping<T>(target));
	}

	template <typename T> void AddMapping(const std::string& key, std::vector<T>* target)
	{
		this->AddMapping<std::vector<T>>(key,
										 ClValueMapping<std::vector<T>>(ClValueTypes(ClValueType::Array),
																		ValueTypesForTemplateType<T>::types,
																		target));
	}

	bool ReadObject(ClReader& reader, ClParseState& state, ClParseStatus& outStatus);

private:
	bool ReadValue(ClReader& reader,
				   ClParseState& state,
				   ClValueMapping<void>* mapping,
				   ClRealType realType,
				   ClValueTypes types,
				   void* target,
				   ClParseStatus& outStatus);
	bool ReadObjectProperties(ClReader& reader, ClParseState& state, ClParseStatus& outStatus);
	bool ReadNumberValue(
		ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus);
	bool ReadStringValue(
		ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus);
	bool ReadIdentifierValue(
		ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus);
	bool ReadBooleanValue(
		ClReader& reader, ClParseState& state, ClValueTypes types, void* target, ClParseStatus& outStatus);
	bool ReadArrayValue(ClReader& reader,
						ClParseState& state,
						ClValueMapping<void>& mapping,
						ClValueTypes types,
						void* target,
						ClParseStatus& outStatus);

	template <typename ValueT>
	bool ReadValueSpecialized(ClReader& reader,
			  ClParseState& state,
			  ClValueMapping<void>& mapping,
			  ClRealType realType,
			  std::vector<ValueT>& vectorRef,
			  ClParseStatus& outStatus)
	{
		vectorRef.resize(vectorRef.size() + 1);
		void* lastItemPtr = reinterpret_cast<void*>(&vectorRef.at(vectorRef.size() - 1));
		return this->ReadValue(reader, state, nullptr, realType, mapping.innerTypes, lastItemPtr, outStatus);
	}

	// special handling for booleans since boolean vectors are bitmasks in disguise
	template <>
	bool ReadValueSpecialized(ClReader& reader,
							  ClParseState& state,
							  ClValueMapping<void>& mapping,
							  ClRealType realType,
							  std::vector<ClBoolean>& vectorRef,
							  ClParseStatus& outStatus)
	{
		ClBoolean outValue;
		if (!this->ReadValue(reader,
							 state,
							 nullptr,
							 realType,
							 mapping.innerTypes,
							 reinterpret_cast<void*>(&outValue),
							 outStatus)) {
			return false;
		}

		vectorRef.push_back(outValue);
		return true;
	}

	template <typename ValueT>
	bool ReadArrayValues(ClReader& reader,
						 ClParseState& state,
						 ClValueMapping<void>& mapping,
						 ClRealType realType,
						 void* targetVoid,
						 ClParseStatus& outStatus)
	{
		std::vector<ValueT>* target = reinterpret_cast<std::vector<ValueT>*>(targetVoid);

		*target = std::vector<ValueT>();
		std::vector<ValueT>& vectorRef = *target;

		static ClRealType valueRealType;
		while (reader.NextArrayValue(state, valueRealType, outStatus)) {
			if (valueRealType != realType) {
				// array value isn't the right type
				outStatus.SetError(
					ClParseStatusType::TypeMismatchError,
					ClParseError(
						fmt::format("expected array value type {} but found {}", realType, valueRealType),
						reader.GetPosition()));
				return false;
			}

			if (!this->ReadValueSpecialized(reader, state, mapping, realType, vectorRef, outStatus)) {
				return false;
			}
		}

		return outStatus.IsOk();
	}

	// specialization to handle identifiers and strings both being std::string
	template <>
	bool ReadArrayValues<std::string>(ClReader& reader,
						 ClParseState& state,
						 ClValueMapping<void>& mapping,
						 ClRealType realType,
						 void* targetVoid,
						 ClParseStatus& outStatus)
	{
		std::vector<std::string>* target = reinterpret_cast<std::vector<std::string>*>(targetVoid);

		*target = std::vector<std::string>();
		std::vector<std::string>& vectorRef = *target;

		static ClRealType valueRealType;
		while (reader.NextArrayValue(state, valueRealType, outStatus)) {
			if (valueRealType != ClRealType::String && valueRealType != ClRealType::Identifier) {
				// array value isn't the right type
				outStatus.SetError(
					ClParseStatusType::TypeMismatchError,
					ClParseError(
						fmt::format("expected array value type {} but found {}", realType, valueRealType),
						reader.GetPosition()));
				return false;
			}

			if (!this->ReadValueSpecialized(reader, state, mapping, valueRealType, vectorRef, outStatus)) {
				return false;
			}
		}

		return outStatus.IsOk();
	}

	bool ReadObjectValue(ClReader& reader,
						 ClParseState& state,
						 ClValueMapping<void>& mapping,
						 ClValueTypes types,
						 void* target,
						 ClParseStatus& outStatus);

	ClParseState& parseState;
	std::map<ClStringTableId, ClValueMapping<void>> mappingTable;
};