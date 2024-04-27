#include "ClTokenizer.h"
#include "ClTokenizer.h"
#include "ClTokenizer.h"
#include "ClTokenizer.h"
#include "ClTokenizer.h"

#include <spdlog/spdlog.h>
#include <fmt/format.h>

ClTokenizer::ClTokenizer(ClParseState& state) :
	text(state.GetInputText())
{
	this->position = 0;
	// skip UTF8 Byte Order Mark if present
	if (this->text.length() > 3 && this->text[0] == '\xEF' && this->text[1] == '\xBB') {
		this->position = 3;
	}
}

bool ClTokenizer::IsDone() const { return this->position >= this->text.length(); }

bool ClTokenizer::NextToken(ClToken& outToken, ClParseStatus& outStatus)
{
	if (this->IsDone()) {
		outStatus.SetStatusType(ClParseStatusType::Ok);
		return false;
	}

	outStatus.SetStatusType(ClParseStatusType::Ok);

	char c = this->text[this->position];
	// skip whitespace
	while (!IsDone() && std::isspace(c)) {
		c = this->text[++this->position];
		// skip comments too
		if (c == '#') {
			while (!IsDone() && c != '\n') {
				c = this->text[++this->position];
			}
		}
	}

	if (IsDone()) {
		outStatus.SetStatusType(ClParseStatusType::Ok);
		return false;
	}

	else if (c == '=') {
		this->FillToken(outToken, ClTokenType::Equals, this->position++, 1);
		return true;
	} else if (c == ':') {
		this->FillToken(outToken, ClTokenType::Colon, this->position++, 1);
		return true;
	} else if (c == '{') {
		this->FillToken(outToken, ClTokenType::OpenBracket, this->position++, 1);
		return true;
	} else if (c == '}') {
		this->FillToken(outToken, ClTokenType::CloseBracket, this->position++, 1);
		return true;
	} else if (c == '>') {
		if (this->IsNextChar('=')) {
			this->FillToken(outToken, ClTokenType::GreaterThanEq, this->position, 2);
			this->position += 2;
		} else {
			this->FillToken(outToken, ClTokenType::GreaterThan, this->position++, 1);
		}
		return true;
	} else if (c == '<') {
		if (this->IsNextChar('=')) {
			this->FillToken(outToken, ClTokenType::LessThanEq, this->position, 2);
			this->position += 2;
		} else {
			this->FillToken(outToken, ClTokenType::LessThan, this->position++, 1);
		}
		return true;
	} else if (c == '?') {
		if (this->IsNextChar('=')) {
			this->FillToken(outToken, ClTokenType::ExistenceCheck, this->position, 2);
			this->position += 2;
			return true;
		}

		outStatus.SetError(ClParseStatusType::TokenizerError,
						   ClParseError("unexpected char ?", this->position));
		return false;
	} else if (c == '-' || std::isdigit(c)) {
		// handle numbers

		int numDigits = c == '-' ? 0 : 1;
		const int startPos = this->position;
		int foundDecimalPlace = -1;
		this->position++;
		while (!this->IsDone()) {
			const char& numC = this->text[this->position];
			if (numC == '.') {
				// 0.05.0, -.5, and .05 are all considered invalid numbers here
				if (foundDecimalPlace != -1 || numDigits < 1) {
					outStatus.SetError(ClParseStatusType::TokenizerError,
									   ClParseError("unexpected char .", this->position));
					return false;
				}

				foundDecimalPlace = this->position;
			} else if (std::isdigit(numC)) {
				numDigits++;
			} else {
				break;
			}

			this->position++;
		}

		// a bare - isn't allowed, and neither is 15. as a number
		if (numDigits < 1 || (this->position - foundDecimalPlace) < 2) {
			outStatus.SetError(ClParseStatusType::TokenizerError,
							   ClParseError("unexpected end of number", this->position));
			return false;
		}

		this->FillToken(outToken, ClTokenType::Number, startPos, this->position - startPos + 1);
		this->position++;
		return true;
	} else if (c == '"') {
		// read a string
		const int startPos = this->position;
		do {
			this->position++;
		} while (!this->IsDone() && this->text[this->position] != '"');

		if (this->IsDone()) {
			outStatus.SetError(ClParseStatusType::TokenizerError,
							   ClParseError("unexpected end of file while reading string", this->position));
			return false;
		}

		this->FillToken(outToken, ClTokenType::String, startPos + 1, this->position - startPos - 1);
		this->position++;
		return true;
	} else if (c == '_' || std::isalnum(c)) {
		const int startPos = this->position;
		do {
			this->position++;
		} while (!this->IsDone()
				 && (this->text[this->position] == '_' || std::isalnum(this->text[this->position])));

		const int length = this->position - startPos;
		if ((length == 3 && this->text[startPos + 0] == 'y' && this->text[startPos + 1] == 'e'
			 && this->text[startPos + 2] == 's')
			|| (length == 2 && this->text[startPos + 0] == 'n' && this->text[startPos + 1] == 'o')) {
			this->FillToken(outToken, ClTokenType::Boolean, startPos, length);
		} else {
			this->FillToken(outToken, ClTokenType::Identifier, startPos, length);
		}
		this->position++;
		return true;
	}

	outStatus.SetError(ClParseStatusType::TokenizerError,
					   ClParseError(fmt::format("unexpected character {} in input", c), this->position));
	return false;
}

bool ClTokenizer::PeekToken(ClToken& outToken, ClParseStatus& outStatus)
{
	const int currentPosition = this->position;
	const bool result = this->NextToken(outToken, outStatus);
	this->position = currentPosition;
	return result;
}

bool ClTokenizer::PeekToToken(uint32_t offset, ClToken& outToken, ClParseStatus& outStatus)
{
	const int currentPosition = this->position;
	for (uint32_t i = 0; i < offset; i++) {
		if (!this->NextToken(outToken, outStatus)) {
			return false;
		}
	}
	this->position = currentPosition;
	return true;
}

void ClTokenizer::FillToken(ClToken& outToken, ClTokenType type, int startPos, int length) const
{
	outToken.type = type;
	outToken.segmentStartIndex = startPos;
	outToken.segmentLength = length;
}

bool ClTokenizer::IsNextChar(char c)
{
	if (this->position > this->text.length() - 1) {
		return false;
	}

	return this->text[this->position + 1] == c;
}

const int ClTokenizer::GetPosition() const { return this->position; }

void ClTokenizer::PrintToken(ClToken& token)
{
	spdlog::info("token type {}, start {}, len {}, text {}",
				 token.type,
				 token.segmentStartIndex,
				 token.segmentLength,
				 this->GetSegmentForToken(token));
}