#pragma once
#include "ASTNode.h"
#include "Tokenizer.h"
#include <memory>
#include <string>
#include <unordered_map>

namespace AIParser {

// 语法分析器
class Parser {
public:
  Parser(const std::string &source);

  std::shared_ptr<ASTNode> parse();

private:
  int current_ast_index = 0;
  Tokenizer tokenizer;
  Token currentToken;

  void advance();

  // 解析方法
  std::shared_ptr<ASTNode> parseExpression();
  std::shared_ptr<ASTNode> parsePrimary();
  std::shared_ptr<ASTNode> parseFunctionCall();
  std::shared_ptr<ASTNode> parseSelector();
  std::shared_ptr<ASTNode> parseSequence();
  std::shared_ptr<ASTNode> parseRepeat();
  std::shared_ptr<ASTNode> parseIf();
  std::shared_ptr<ASTNode> parseValue();
  std::shared_ptr<ASTNode> parseCondition();

  // 工具方法
  std::vector<std::shared_ptr<ASTNode>> parseArgumentList();
  bool match(TokenType type);
  bool check(TokenType type) const;
  void consume(TokenType type, const std::string &errorMsg);

  // 关键字映射
  static const std::unordered_map<std::string, NodeType> controlKeywords;
};

} // namespace AIParser