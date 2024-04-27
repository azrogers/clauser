#pragma once

#include <spdlog/spdlog.h>

enum class ClParseStatusType {
	/**
	 * No parse errors, keep going.
	 */
	Ok,
	/**
	 * Failed to tokenize string.
	 */
	TokenizerError,
	/**
	 * An operation was attempted that doesn't match the parser's internal state - for example, calling
	 * EndReadObject on a ClReader without calling BeginReadObject.
	 */
	StateMismatchError,
	/**
	 * The reader expected to find a token of a certain type but instead found another.
	 */
	UnexpectedTokenError,
	/**
	 * An operation to read a number failed because the number was an invalid format.
	 */
	InvalidNumberError,
	/**
	 * An unknown key was encountered while reading an object.
	 */
	UnknownKeyError,
	/**
	 * File contains a different type than what was expected.
	 */
	TypeMismatchError,
	/**
	 * Something that isn't currently supported.
	 */
	Unsupported,
	/**
	 * The parser is in an invalid state (for example, an invalid pointer)
	 */
	InvalidState
};

template <> struct fmt::formatter<ClParseStatusType> : formatter<string_view> {
	template <typename FormatContext> auto format(ClParseStatusType type, FormatContext& ctx)
	{
		string_view name = "<missing enum entry>";
		switch (type) {
		case ClParseStatusType::Ok:
			name = "Ok";
			break;
		case ClParseStatusType::TokenizerError:
			name = "TokenizerError";
			break;
		case ClParseStatusType::StateMismatchError:
			name = "StateMismatchError";
			break;
		case ClParseStatusType::UnexpectedTokenError:
			name = "UnexpectedTokenError";
			break;
		case ClParseStatusType::InvalidNumberError:
			name = "InvalidNumberError";
			break;
		case ClParseStatusType::UnknownKeyError:
			name = "UnknownKeyError";
			break;
		case ClParseStatusType::TypeMismatchError:
			name = "TypeMismatchError";
			break;
		case ClParseStatusType::Unsupported:
			name = "Unsupported";
			break;
		case ClParseStatusType::InvalidState:
			name = "InvalidState";
			break;
		}
		return formatter<string_view>::format(name, ctx);
	}
};

class ClParseError {
public:
	ClParseError() :
		position(0)
	{
	}

	ClParseError(const std::string& message, int position) :
		message(message),
		position(position)
	{
	}

	void Log() { spdlog::error("Clauser parse error at position {}: {}", this->position, this->message); }

private:
	std::string message;
	int position;
};

class ClParseStatus {
public:
	ClParseStatus() { }

	bool IsOk() { return this->type == ClParseStatusType::Ok; }
	ClParseStatusType GetStatusType() { return this->type; }
	ClParseError& GetError() { return this->error; }

	void SetStatusType(ClParseStatusType status) { this->type = status; }
	void SetError(ClParseStatusType status, ClParseError&& newError)
	{
		this->type = status;
		this->error = std::move(newError);
	}

private:
	ClParseStatusType type = ClParseStatusType::Ok;
	ClParseError error;
};