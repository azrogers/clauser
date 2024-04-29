#pragma once

#include <stack>
#include <string_view>
#include <charconv>
#include <string_view>

#include "ClTypes.h"
#include "ClParseState.h"
#include "ClTokenizer.h"

enum class ClReaderState { Object, Array };

template <> struct fmt::formatter<ClReaderState> : formatter<string_view> {
    template <typename FormatContext> auto format(ClReaderState type, FormatContext& ctx)
    {
        string_view name = "<missing enum entry>";
        switch (type) {
        case ClReaderState::Object:
            name = "Object";
            break;
        case ClReaderState::Array:
            name = "Array";
            break;
        }
        return formatter<string_view>::format(name, ctx);
    }
};

class ClReader {
public:
    ClReader(ClParseState& state);

    bool PeekIdentifier(ClParseState& state, ClStringTableId& outIdentifier, ClParseStatus& outStatus);
    bool TryPeekIdentifier(ClParseState& state, ClStringTableId& outIdentifier, ClParseStatus& outStatus);

    bool BeginReadObject(ClParseState& state, ClParseStatus& outStatus);
    bool EndReadObject(ClParseState& state, ClParseStatus& outStatus);

    bool NextProperty(ClParseState& state,
                      ClStringTableId& outKey,
                      ClRealType& outPropertyType,
                      ClParseStatus& outStatus);

    inline bool ReadInteger(ClParseState& state, ClInteger& outValue, ClParseStatus& outStatus)
    {
        return this->ReadNumber<ClInteger>(state, outValue, outStatus);
    }
    inline bool ReadInteger64(ClParseState& state, ClInteger64& outValue, ClParseStatus& outStatus)
    {
        return this->ReadNumber<ClInteger64>(state, outValue, outStatus);
    }

    inline bool ReadDecimal(ClParseState& state, ClDecimal& outValue, ClParseStatus& outStatus)
    {
        return this->ReadNumber<ClDecimal>(state, outValue, outStatus);
    }
    inline bool ReadDecimal64(ClParseState& state, ClDecimal64& outValue, ClParseStatus& outStatus)
    {
        return this->ReadNumber<ClDecimal64>(state, outValue, outStatus);
    }

    bool ReadString(ClParseState& state, ClStringTableId& outValue, ClParseStatus& outStatus);
    bool ReadIdentifier(ClParseState& state, ClStringTableId& outValue, ClParseStatus& outStatus);
    bool ReadBoolean(ClParseState& state, ClBoolean& outValue, ClParseStatus& outStatus);

    bool BeginReadArray(ClParseState& state, ClParseStatus& outStatus);
    bool NextArrayValue(ClParseState& state, ClRealType& outType, ClParseStatus& outStatus);
    bool EndReadArray(ClParseState& state, ClParseStatus& outStatus);

    inline int GetPosition() const { return this->tokenizer.GetPosition(); }

    bool
    RealTypeFromTokenType(ClTokenType tokenType, ClRealType& outRealType, ClParseStatus& outStatus) const;

private:
    template <typename NumberType>
    bool ReadNumber(ClParseState& state, NumberType& outValue, ClParseStatus& outStatus)
    {
        static ClToken token;
        if (!this->ExpectToken(ClTokenType::Number, token, outStatus)) {
            return false;
        }

        // attempt to parse number
        std::string_view segment = this->tokenizer.GetSegmentForToken(token);
        std::from_chars_result result =
            std::from_chars(segment.data(), segment.data() + segment.length(), outValue);
        if (result.ec == std::errc::invalid_argument) {
            outStatus.SetError(ClParseStatusType::InvalidNumberError,
                               ClParseError(fmt::format("failed to parse number from token '{}'", segment),
                                            this->tokenizer.GetPosition()));
            return false;
        }

        outStatus.SetStatusType(ClParseStatusType::Ok);
        return true;
    }

    void PushReaderState(ClReaderState newState);
    bool PopReaderState(ClReaderState expectedState, ClParseStatus& outStatus);
    bool ExpectState(ClReaderState expectedState, ClParseStatus& outStatus);

    bool ExpectToken(ClTokenType expectedType, ClToken& outToken, ClParseStatus& outStatus);
    bool ExpectTokenPeek(ClTokenType expectedType, ClToken& outToken, ClParseStatus& outStatus);

    ClTokenizer tokenizer;
    // top level is basically an anonymous object
    ClReaderState currentState = ClReaderState::Object;
    std::stack<ClReaderState> stateStack;

    bool isAwaitingValue = false;
};