
#pragma once
#include <cctype>
#include <string>
#include <vector>

namespace AIParser {

// Token类型
enum class TokenType {
  IDENTIFIER, // 标识符 (函数名、变量名)
  NUMBER,     // 数字
  STRING,     // 字符串
  LPAREN,     // (
  RPAREN,     // )
  COMMA,      // ,
  OPERATOR,   // < > = 等
  BOOL,       // true/false
  END         // 结束
};

// Token结构
struct Token {
  TokenType type;
  std::string value;
  int line;
  int column;

  // 添加默认构造函数
  Token() : type(TokenType::END), value(""), line(0), column(0) {}
  
  Token(TokenType t, const std::string &v = "", int l = 0, int c = 0)
      : type(t), value(v), line(l), column(c) {}
};

// 词法分析器
class Tokenizer {
public:
  Tokenizer(const std::string &source);

  Token nextToken();
  Token peekToken();
  bool hasMoreTokens() const;

private:
  std::string source;
  size_t position;
  int line;
  int column;

  char currentChar() const;
  char peekChar() const;
  void skipWhitespace();
  void skipComment();
  void advance();
  Token readIdentifier();
  Token readNumber();
  Token readString();
  Token readOperator();
};

} // namespace AIParser