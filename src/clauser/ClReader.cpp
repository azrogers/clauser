#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"
#include "ClReader.h"

#include <fmt/format.h>

using namespace std;

ClReader::ClReader(ClParseState& state) :
    tokenizer(state)
{
}

bool ClReader::PeekIdentifier(ClParseState& state, ClStringTableId& outIdentifier, ClParseStatus& outStatus)
{
    static ClToken token;
    const bool result = this->ExpectTokenPeek(ClTokenType::Identifier, token, outStatus);
    if (!result) {
        return false;
    }

    outIdentifier = state.AddString(this->tokenizer.GetSegmentForToken(token));
    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::TryPeekIdentifier(ClParseState& state,
                                 ClStringTableId& outIdentifier,
                                 ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->tokenizer.PeekToken(token, outStatus)) {
        return false;
    }

    outStatus.SetStatusType(ClParseStatusType::Ok);
    if (token.GetType() != ClTokenType::Identifier) {
        return false;
    }

    outIdentifier = state.AddString(this->tokenizer.GetSegmentForToken(token));
    return true;
}

bool ClReader::BeginReadObject(ClParseState& state, ClParseStatus& outStatus)
{
    if (!this->ExpectState(ClReaderState::Object, outStatus)) {
        return false;
    }

    static ClToken token;

    // try to consume = {
    if (!this->ExpectToken(ClTokenType::OpenBracket, token, outStatus)) {
        return false;
    }

    this->PushReaderState(ClReaderState::Object);
    outStatus.SetStatusType(ClParseStatusType::Ok);

    return true;
}

bool ClReader::EndReadObject(ClParseState& state, ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->ExpectToken(ClTokenType::CloseBracket, token, outStatus)) {
        return false;
    }

    if (!this->PopReaderState(ClReaderState::Object, outStatus)) {
        return false;
    }

    return true;
}

bool ClReader::NextProperty(ClParseState& state,
                            ClStringTableId& outKey,
                            ClRealType& outPropertyType,
                            ClParseStatus& outStatus)
{
    if (!this->ExpectState(ClReaderState::Object, outStatus)) {
        return false;
    }

    static ClToken token;
    if (!this->tokenizer.NextToken(token, outStatus)) {
        if (outStatus.IsOk() && this->stateStack.empty()) {
            // EOF on the root object, we this is a valid end of the object
            outStatus.SetStatusType(ClParseStatusType::Ok);
        }
        return false;
    }

    // we've reached the end, so we haven't hit an error but we are done
    if (token.GetType() == ClTokenType::CloseBracket && !this->stateStack.empty()) {
        outStatus.SetStatusType(ClParseStatusType::Ok);
        return false;
    }

    if (token.GetType() != ClTokenType::Identifier) {
        outStatus.SetError(
            ClParseStatusType::UnexpectedTokenError,
            ClParseError(fmt::format("unexpected token type {}, expected Identifier", token.GetType()),
                         token.GetPosition()));
        return false;
    }

    outKey = state.AddString(this->tokenizer.GetSegmentForToken(token));
    if (!this->ExpectToken(ClTokenType::Equals, token, outStatus)) {
        return false;
    }

    if (!this->tokenizer.PeekToken(token, outStatus)) {
        return false;
    }

    if (!this->RealTypeFromTokenType(token.GetType(), outPropertyType, outStatus)) {
        return false;
    }

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::ReadString(ClParseState& state, ClStringTableId& outValue, ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->ExpectToken(ClTokenType::String, token, outStatus)) {
        return false;
    }

    outValue = state.AddString(this->tokenizer.GetSegmentForToken(token));

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::ReadIdentifier(ClParseState& state, ClStringTableId& outValue, ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->ExpectToken(ClTokenType::Identifier, token, outStatus)) {
        return false;
    }

    outValue = state.AddString(this->tokenizer.GetSegmentForToken(token));

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::ReadBoolean(ClParseState& state, ClBoolean& outValue, ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->ExpectToken(ClTokenType::Boolean, token, outStatus)) {
        return false;
    }

    std::string_view segment = this->tokenizer.GetSegmentForToken(token);
    // value is either 'yes' or 'no'
    outValue = segment.at(0) == 'y';

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::BeginReadArray(ClParseState& state, ClParseStatus& outStatus)
{
    if (!this->ExpectState(ClReaderState::Object, outStatus)) {
        return false;
    }

    static ClToken token;
    if (!this->ExpectToken(ClTokenType::OpenBracket, token, outStatus)) {
        return false;
    }

    this->PushReaderState(ClReaderState::Array);
    outStatus.SetStatusType(ClParseStatusType::Ok);

    return true;
}

bool ClReader::NextArrayValue(ClParseState& state, ClRealType& outType, ClParseStatus& outStatus)
{
    if (!this->ExpectState(ClReaderState::Array, outStatus)) {
        return false;
    }

    static ClToken token;
    if (!this->tokenizer.PeekToken(token, outStatus)) {
        return false;
    }

    // we've hit the end of the array
    if (token.GetType() == ClTokenType::CloseBracket) {
        outStatus.SetStatusType(ClParseStatusType::Ok);
        return false;
    }

    if (!this->RealTypeFromTokenType(token.GetType(), outType, outStatus)) {
        return false;
    }

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::EndReadArray(ClParseState& state, ClParseStatus& outStatus)
{
    static ClToken token;
    if (!this->ExpectToken(ClTokenType::CloseBracket, token, outStatus)) {
        return false;
    }

    if (!this->PopReaderState(ClReaderState::Array, outStatus)) {
        return false;
    }

    return true;
}

bool ClReader::RealTypeFromTokenType(ClTokenType tokenType,
                                     ClRealType& outRealType,
                                     ClParseStatus& outStatus) const
{

    switch (tokenType) {
    case ClTokenType::Boolean:
        outRealType = ClRealType::Boolean;
        break;
    case ClTokenType::Number:
        outRealType = ClRealType::Number;
        break;
    case ClTokenType::Identifier:
        outRealType = ClRealType::Identifier;
        break;
    case ClTokenType::String:
        outRealType = ClRealType::String;
        break;
    case ClTokenType::OpenBracket:
        outRealType = ClRealType::ObjectOrArray;
        break;
    default:
        outStatus.SetError(ClParseStatusType::UnexpectedTokenError,
                           ClParseError(fmt::format("invalid token {} in property value", tokenType),
                                        this->tokenizer.GetPosition()));
        return false;
    }

    return true;
}

void ClReader::PushReaderState(ClReaderState newState)
{
    this->stateStack.push(this->currentState);
    this->currentState = newState;
}

bool ClReader::PopReaderState(ClReaderState expectedState, ClParseStatus& outStatus)
{
    if (this->currentState != expectedState) {
        outStatus.SetError(
            ClParseStatusType::StateMismatchError,
            ClParseError(fmt::format("tried to end Object but state was {}", this->currentState),
                         this->tokenizer.GetPosition()));
        return false;
    }

    if (this->stateStack.empty()) {
        outStatus.SetError(ClParseStatusType::StateMismatchError,
                           ClParseError("EndReadObject called without matching BeginReadObject",
                                        this->tokenizer.GetPosition()));
        return false;
    }

    this->currentState = this->stateStack.top();
    this->stateStack.pop();

    return true;
}

bool ClReader::ExpectState(ClReaderState expectedState, ClParseStatus& outStatus)
{
    if (this->currentState != expectedState) {
        outStatus.SetError(
            ClParseStatusType::StateMismatchError,
            ClParseError(fmt::format("expected state {}, found state {}", expectedState, this->currentState),
                         this->tokenizer.GetPosition()));
        return false;
    }

    outStatus.SetStatusType(ClParseStatusType::Ok);
    return true;
}

bool ClReader::ExpectToken(ClTokenType expectedType, ClToken& outToken, ClParseStatus& outStatus)
{
    if (!this->tokenizer.NextToken(outToken, outStatus)) {
        return false;
    }

    if (outToken.GetType() != expectedType) {
        outStatus.SetError(
            ClParseStatusType::UnexpectedTokenError,
            ClParseError(
                fmt::format("unexpected token type {}, expected {}", outToken.GetType(), expectedType),
                outToken.GetPosition()));
        return false;
    }

    return true;
}

bool ClReader::ExpectTokenPeek(ClTokenType expectedType, ClToken& outToken, ClParseStatus& outStatus)
{
    if (!this->tokenizer.PeekToken(outToken, outStatus)) {
        return false;
    }

    if (outToken.GetType() != expectedType) {
        outStatus.SetError(
            ClParseStatusType::UnexpectedTokenError,
            ClParseError(
                fmt::format("unexpected token type {}, expected {}", outToken.GetType(), expectedType),
                outToken.GetPosition()));
        return false;
    }

    return true;
}