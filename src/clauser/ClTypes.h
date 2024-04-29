#pragma once

#include <cstdint>
#include <fmt/format.h>

/**
 * Strings are de-duplicated and stored in the \ref ClParseState string table.
 You can refer to them using this ID.
 */
typedef uint32_t ClStringTableId;

typedef int32_t ClInteger;
typedef int64_t ClInteger64;
typedef float ClDecimal;
typedef double ClDecimal64;
typedef bool ClBoolean;

enum class ClValueType : uint8_t {
    Invalid = 0,
    String = 1,
    Identifier = 2,
    Integer = 3,
    Integer64 = 4,
    Decimal = 5,
    Decimal64 = 6,
    Object = 7,
    Array = 8,
    Boolean = 9
};

template <> struct fmt::formatter<ClValueType> : formatter<string_view> {
    template <typename FormatContext> auto format(ClValueType type, FormatContext& ctx)
    {
        string_view name = "<missing enum entry>";
        switch (type) {

        case ClValueType::Invalid:
            name = "Invalid";
            break;
        case ClValueType::String:
            name = "String";
            break;
        case ClValueType::Identifier:
            name = "Identifier";
            break;
        case ClValueType::Integer:
            name = "Integer";
            break;
        case ClValueType::Integer64:
            name = "Integer64";
            break;
        case ClValueType::Decimal:
            name = "Decimal";
            break;
        case ClValueType::Decimal64:
            name = "Decimal64";
            break;
        case ClValueType::Object:
            name = "Object";
            break;
        case ClValueType::Array:
            name = "Array";
            break;
        case ClValueType::Boolean:
            name = "Boolean";
            break;
        }
        return formatter<string_view>::format(name, ctx);
    }
};

/**
 * Types that are actually differentiable purely from tokens.
 */
enum class ClRealType { ObjectOrArray, Number, Boolean, String, Identifier };

template <> struct fmt::formatter<ClRealType> : formatter<string_view> {
    template <typename FormatContext> auto format(ClRealType type, FormatContext& ctx)
    {
        string_view name = "<missing enum entry>";
        switch (type) {
        case ClRealType::ObjectOrArray:
            name = "ObjectOrArray";
            break;
        case ClRealType::Number:
            name = "Number";
            break;
        case ClRealType::Boolean:
            name = "Boolean";
            break;
        case ClRealType::String:
            name = "String";
            break;
        case ClRealType::Identifier:
            name = "Identifier";
            break;
        }
        return formatter<string_view>::format(name, ctx);
    }
};