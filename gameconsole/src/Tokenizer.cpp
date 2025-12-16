#include "AIParser/Tokenizer.h"
#include <stdexcept>

namespace AIParser {

Tokenizer::Tokenizer(const std::string &source)
    : source(source), position(0), line(1), column(1) {}

char Tokenizer::currentChar() const {
  if (position >= source.length())
    return '\0';
  return source[position];
}

char Tokenizer::peekChar() const {
  if (position + 1 >= source.length())
    return '\0';
  return source[position + 1];
}

void Tokenizer::skipWhitespace() {
  while (currentChar() != '\0') {
    if (currentChar() == ' ' || currentChar() == '\t' ||
        currentChar() == '\r') {
      advance();
    } else if (currentChar() == '\n') {
      line++;
      column = 1;
      position++;
    } else if (currentChar() == '#') {
      skipComment();
    } else {
      break;
    }
  }
}

void Tokenizer::skipComment() {
  while (currentChar() != '\0' && currentChar() != '\n') {
    advance();
  }
}

Token Tokenizer::nextToken() {
  skipWhitespace();

  if (currentChar() == '\0') {
    return Token(TokenType::END, "", line, column);
  }

  char c = currentChar();

  // 标识符或关键字
  if (std::isalpha(c) || c == '_') {
    return readIdentifier();
  }

  // 数字
  if (std::isdigit(c) || c == '-') {
    return readNumber();
  }

  // 字符串
  if (c == '"' || c == '\'') {
    return readString();
  }

  // 操作符
  if (c == '<' || c == '>' || c == '=' || c == '!') {
    return readOperator();
  }

  // 括号和逗号
  if (c == '(') {
    advance();
    return Token(TokenType::LPAREN, "(", line, column - 1);
  }
  if (c == ')') {
    advance();
    return Token(TokenType::RPAREN, ")", line, column - 1);
  }
  if (c == ',') {
    advance();
    return Token(TokenType::COMMA, ",", line, column - 1);
  } else {
    return Token(TokenType::COMMA, ",", line, column - 1);
  }

  // throw std::runtime_error("Unexpected character: " + std::string(1, c));
}

Token Tokenizer::peekToken() {
  size_t savedPos = position;
  int savedLine = line;
  int savedCol = column;

  Token token = nextToken();

  position = savedPos;
  line = savedLine;
  column = savedCol;

  return token;
}

bool Tokenizer::hasMoreTokens() const { return position < source.length(); }

void Tokenizer::advance() {
  if (currentChar() != '\0') {
    position++;
    column++;
  }
}

Token Tokenizer::readIdentifier() {
  int startLine = line;
  int startCol = column;
  std::string value;

  while (std::isalnum(currentChar()) || currentChar() == '_') {
    value += currentChar();
    advance();
  }

  // 检查是否为布尔值
  if (value == "true" || value == "false") {
    return Token(TokenType::BOOL, value, startLine, startCol);
  }

  return Token(TokenType::IDENTIFIER, value, startLine, startCol);
}

Token Tokenizer::readNumber() {
  int startLine = line;
  int startCol = column;
  std::string value;
  bool hasDot = false;

  // 处理负号
  if (currentChar() == '-') {
    value += currentChar();
    advance();
  }

  while (std::isdigit(currentChar()) || currentChar() == '.') {
    if (currentChar() == '.') {
      if (hasDot)
        break;
      hasDot = true;
    }
    value += currentChar();
    advance();
  }

  return Token(TokenType::NUMBER, value, startLine, startCol);
}

Token Tokenizer::readString() {
  int startLine = line;
  int startCol = column;
  char quote = currentChar();
  std::string value;

  advance(); // 跳过开始引号

  while (currentChar() != '\0' && currentChar() != quote) {
    // 处理转义字符
    if (currentChar() == '\\') {
      advance();
      switch (currentChar()) {
      case 'n':
        value += '\n';
        break;
      case 't':
        value += '\t';
        break;
      case 'r':
        value += '\r';
        break;
      case '\\':
        value += '\\';
        break;
      case '"':
        value += '"';
        break;
      case '\'':
        value += '\'';
        break;
      default:
        value += currentChar();
        break;
      }
    } else {
      value += currentChar();
    }
    advance();
  }

  if (currentChar() != quote) {
    return Token(TokenType::STRING, value, startLine, startCol);
    // throw std::runtime_error("Unterminated string literal");
  }

  advance(); // 跳过结束引号

  return Token(TokenType::STRING, value, startLine, startCol);
}

Token Tokenizer::readOperator() {
  int startLine = line;
  int startCol = column;
  std::string value;

  value += currentChar();
  advance();

  // 处理双字符操作符
  if ((value[0] == '<' || value[0] == '>' || value[0] == '=' ||
       value[0] == '!') &&
      currentChar() == '=') {
    value += currentChar();
    advance();
  }

  return Token(TokenType::OPERATOR, value, startLine, startCol);
}

} // namespace AIParser