#pragma once

#include <string>
#include <string_view>

#include "ClParseStatus.h"
#include "ClParseState.h"

/**
 * The type of a token in a Clausewitz source file.
 */
enum class ClTokenType {
	/**
	 * Tokens should never be invalid!
	 */
	Invalid,
	/**
	 * A non-string bit of text.
	 */
	Identifier,
	/**
	 * An integer or decimal number.
	 */
	Number,
	/**
	 * A string wrapped in double quotes.
	 */
	String,
	/**
	 * The '=' symbol.
	 */
	Equals,
	/**
	 * The ':' symbol.
	 */
	Colon,
	/**
	 * The '{' symbol.
	 */
	OpenBracket,
	/**
	 * The '}' symbol.
	 */
	CloseBracket,
	/**
	 * The '>' symbol.
	 */
	GreaterThan,
	/**
	 * The '<' symbol.
	 */
	LessThan,
	/**
	 * The '>=' symbol.
	 */
	GreaterThanEq,
	/**
	 * The '<=' symbol.
	 */
	LessThanEq,
	/**
	 * The '?=' symbol.
	 */
	ExistenceCheck,
	/**
	 * A yes or no value.
	 */
	Boolean,
};

template <> struct fmt::formatter<ClTokenType> : formatter<string_view> {
	template <typename FormatContext> auto format(ClTokenType type, FormatContext& ctx)
	{
		string_view name = "<missing enum entry>";
		switch (type) {
		case ClTokenType::Invalid:
			name = "Invalid";
			break;
		case ClTokenType::Equals:
			name = "Equals";
			break;
		case ClTokenType::String:
			name = "String";
			break;
		case ClTokenType::Identifier:
			name = "Identifier";
			break;
		case ClTokenType::Number:
			name = "Number";
			break;
		case ClTokenType::Colon:
			name = "Colon";
			break;
		case ClTokenType::OpenBracket:
			name = "OpenBracket";
			break;
		case ClTokenType::CloseBracket:
			name = "CloseBracket";
			break;
		case ClTokenType::GreaterThan:
			name = "GreaterThan";
			break;
		case ClTokenType::LessThan:
			name = "LessThan";
			break;
		case ClTokenType::GreaterThanEq:
			name = "GreaterThanEq";
			break;
		case ClTokenType::LessThanEq:
			name = "LessThanEq";
			break;
		case ClTokenType::ExistenceCheck:
			name = "ExistenceCheck";
			break;
		case ClTokenType::Boolean:
			name = "Boolean";
			break;
		}
		return formatter<string_view>::format(name, ctx);
	}
};

/**
 * A single token in a Clausewitz source file.
 */
class ClToken {
public:
	ClToken() :
		type(ClTokenType::Invalid),
		segmentStartIndex(0),
		segmentLength(0)
	{
	}

	constexpr ClTokenType GetType() const { return this->type; }
	constexpr int GetPosition() const { return this->segmentStartIndex; }

private:
	ClTokenType type;
	int segmentStartIndex;
	int segmentLength;

	friend class ClTokenizer;
};

/**
 * Tokenizes a Clausewitz source file.
 */
class ClTokenizer {
public:
	/**
	 * Creates a new tokenizer from an input string.
	 */
	ClTokenizer(ClParseState& state);

	/**
	 * Obtains the next token in the input text.
	 */
	bool NextToken(ClToken& outToken, ClParseStatus& outStatus);
	/**
	 * Equivalent to \ref NextToken, but doesn't change the tokenizer's internal position.
	 */
	bool PeekToken(ClToken& outToken, ClParseStatus& outStatus);
	/**
	 * Peek \ref offset tokens ahead of the current position, if available.
	 */
	bool PeekToToken(uint32_t offset, ClToken& outToken, ClParseStatus& outStatus);
	/**
	 * If true, the last token in the input has been reached.
	 */
	bool IsDone() const;
	/**
	 * Returns the tokenizer's current position in the input text.
	 */
	const int GetPosition() const;
	/**
	 * Returns an \ref std::string_view pointing to the range of text this token represents.
	 */
	constexpr const std::string_view GetSegmentForToken(const ClToken& token) const
	{
		return std::string_view(this->text.data() + token.segmentStartIndex, token.segmentLength);
	}

	void PrintToken(ClToken& token);

private:
	void FillToken(ClToken& outToken, ClTokenType type, int startPos, int length) const;
	bool IsNextChar(char c);

	int position = 0;

	const std::string& text;
};