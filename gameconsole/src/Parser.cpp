#include "AIParser/Parser.h"
#include "AIParser/AIActions.h"
// #include "godot_cpp/core/error_macros.hpp"
// #include "wrappers.h"
#include <algorithm>
#include <stdexcept>

namespace AIParser {

const std::unordered_map<std::string, NodeType> Parser::controlKeywords = {
    {"selector", NodeType::SELECTOR},
    {"sequence", NodeType::SEQUENCE},
    {"if", NodeType::IF},
    {"repeat", NodeType::REPEAT}};

Parser::Parser(const std::string &source) : tokenizer(source) { advance(); }

void Parser::advance() { currentToken = tokenizer.nextToken(); }

bool Parser::match(TokenType type) {
  if (check(type)) {
    advance();
    return true;
  }
  return false;
}

bool Parser::check(TokenType type) const { return currentToken.type == type; }

void Parser::consume(TokenType type, const std::string &errorMsg) {
  if (!check(type)) {
    return;
    // throw std::runtime_error(errorMsg + ", got: " + currentToken.value);
  }
  advance();
}

std::shared_ptr<ASTNode> Parser::parse() {
  current_ast_index = 0;
  return parseExpression();
}

std::shared_ptr<ASTNode> Parser::parseExpression() {
  if (check(TokenType::IDENTIFIER)) {
    std::string identifier = currentToken.value;

    // 检查是否为控制流关键字
    auto it = controlKeywords.find(identifier);
    if (it != controlKeywords.end()) {
      advance(); // 消耗标识符
      switch (it->second) {
      case NodeType::SELECTOR:
        return parseSelector();
      case NodeType::SEQUENCE:
        return parseSequence();
      case NodeType::IF:
        return parseIf();
      case NodeType::REPEAT:
        return parseRepeat();
      default:
        return parseSelector();
        // throw std::runtime_error("Unimplemented control keyword: " +
        //                          identifier);
      }
    }

    // 普通函数调用
    return parseFunctionCall();
  }

  // 值或变量
  return parsePrimary();
}

std::shared_ptr<ASTNode> Parser::parseSelector() {
  consume(TokenType::LPAREN, "Expected '(' after selector");

  std::vector<std::shared_ptr<ASTNode>> children;

  while (!check(TokenType::RPAREN)) {
    children.push_back(parseExpression());

    if (!check(TokenType::RPAREN)) {
      consume(TokenType::COMMA, "Expected ',' or ')' in selector arguments");
    }
  }

  consume(TokenType::RPAREN, "Expected ')' after selector arguments");

  current_ast_index += 1;
  auto node = std::make_shared<SelectorNode>(children);
  node->treeIndex = current_ast_index;
  return node;
}

std::shared_ptr<ASTNode> Parser::parseSequence() {
  consume(TokenType::LPAREN, "Expected '(' after sequence");

  std::vector<std::shared_ptr<ASTNode>> children;

  while (!check(TokenType::RPAREN)) {
    children.push_back(parseExpression());

    if (!check(TokenType::RPAREN)) {
      consume(TokenType::COMMA, "Expected ',' or ')' in sequence arguments");
    }
  }

  consume(TokenType::RPAREN, "Expected ')' after sequence arguments");

  auto node = std::make_shared<SequenceNode>(children);
  node->child_size = children.size();
  return node;
}

// 添加parseRepeat方法
// TODO, repeat函数中无法正确计算current_index
std::shared_ptr<ASTNode> Parser::parseRepeat() {
  consume(TokenType::LPAREN, "Expected '(' after repeat");

  int prev_index = current_ast_index;
  // 第一个参数：要重复的子节点
  auto child = parseExpression();

  consume(TokenType::COMMA, "Expected ',' after repeat child");

  // 第二个参数：重复次数
  auto count = parseExpression();

  consume(TokenType::RPAREN, "Expected ')' after repeat arguments");

  // 执行完了之后判断当前的index，即可知道一轮有多少次数
  int index_one_turn = current_ast_index - prev_index;
  // 转换为 RepeatNode 指针
  if (auto countValue = std::dynamic_pointer_cast<ValueNode>(count)) {
    int repeatCount = 0;
    if (std::holds_alternative<int>(countValue->value)) {
      repeatCount = std::get<int>(countValue->value);
    } else if (std::holds_alternative<float>(countValue->value)) {
      repeatCount = static_cast<int>(std::get<float>(countValue->value));
    }
    current_ast_index += (repeatCount - 1) * index_one_turn;
  }
  // 对于条件循环，无需设置index，每次恢复检查然后继续条件循环

  return std::make_shared<RepeatNode>(child, count);
  // current_ast_index += count;
  // auto node = std::make_shared<RepeatNode>(child, count);
  // node->treeIndex = current_ast_index;
  // return node;
}

std::shared_ptr<ASTNode> Parser::parseIf() {
  consume(TokenType::LPAREN, "Expected '(' after if");

  // 解析条件
  auto condition = parseCondition();

  consume(TokenType::COMMA, "Expected ',' after if condition");

  // 解析true分支
  auto trueBranch = parseExpression();

  // 解析可选的false分支
  std::shared_ptr<ASTNode> falseBranch = nullptr;
  if (match(TokenType::COMMA)) {
    falseBranch = parseExpression();
  }

  consume(TokenType::RPAREN, "Expected ')' after if arguments");

  current_ast_index += 1;
  auto node = std::make_shared<IfNode>(condition, trueBranch, falseBranch);
  node->treeIndex = current_ast_index;
  return node;
}

std::shared_ptr<ASTNode> Parser::parseCondition() {
  auto left = parsePrimary();

  if (check(TokenType::OPERATOR)) {
    std::string op = currentToken.value;
    advance();

    auto right = parsePrimary();

    // 映射操作符到NodeType
    NodeType opType;
    if (op == "<")
      opType = NodeType::LESS;
    else if (op == ">")
      opType = NodeType::GREATER;
    else if (op == "==")
      opType = NodeType::EQUAL;
    else if (op == "!=")
      opType = NodeType::NOT_EQUAL;
    else if (op == "<=")
      opType = NodeType::LESS_EQUAL;
    else if (op == ">=")
      opType = NodeType::GREATER_EQUAL;
    else
      opType = NodeType::EQUAL;

    return std::make_shared<ConditionNode>(left, opType, right);
  }

  return left;
}

std::shared_ptr<ASTNode> Parser::parseFunctionCall() {
  std::string functionName = currentToken.value;
  advance();

  consume(TokenType::LPAREN, "Expected '(' after function name");

  auto args = parseArgumentList();

  consume(TokenType::RPAREN, "Expected ')' after function arguments");

  if (ActionRegistry::getInstance().isBuiltinAction(functionName)) {
    return std::make_shared<FunctionCallNode>(functionName, args);
  } else {
    current_ast_index += 1;
    auto node = std::make_shared<FunctionCallNode>(functionName, args);
    node->treeIndex = current_ast_index;
    return node;
  }
}

std::shared_ptr<ASTNode> Parser::parsePrimary() {
  if (check(TokenType::NUMBER)) {
    return parseValue();
  } else if (check(TokenType::STRING)) {
    return parseValue();
  } else if (check(TokenType::BOOL)) {
    return parseValue();
  } else if (check(TokenType::IDENTIFIER)) {
    // 可能是变量或函数调用
    Token next = tokenizer.peekToken();
    if (next.type == TokenType::LPAREN) {
      return parseFunctionCall();
    } else {
      // 变量
      std::string varName = currentToken.value;
      advance();
      return std::make_shared<VariableNode>(varName);
    }
  } else if (match(TokenType::LPAREN)) {
    auto expr = parseExpression();
    consume(TokenType::RPAREN, "Expected ')' after expression");
    return expr;
  } else {
    // throw std::runtime_error("Unexpected token in primary expression");
    return std::make_shared<VariableNode>("");
  }
}

std::shared_ptr<ASTNode> Parser::parseValue() {
  Token token = currentToken;
  advance();

  if (token.type == TokenType::NUMBER) {
    // 检查是否是浮点数
    if (token.value.find('.') != std::string::npos) {
      float value = std::stof(token.value);
      return std::make_shared<ValueNode>(value);
    } else {
      int value = std::stoi(token.value);
      return std::make_shared<ValueNode>(value);
    }
  } else if (token.type == TokenType::STRING) {
    return std::make_shared<ValueNode>(token.value);
  } else if (token.type == TokenType::BOOL) {
    bool value = (token.value == "true");
    return std::make_shared<ValueNode>(value, NodeType::BOOL);
  } else {
    return std::make_shared<ValueNode>("");
  }
}

std::vector<std::shared_ptr<ASTNode>> Parser::parseArgumentList() {
  std::vector<std::shared_ptr<ASTNode>> args;

  if (check(TokenType::RPAREN)) {
    return args; // 空参数列表
  }

  while (true) {
    args.push_back(parseExpression());

    if (check(TokenType::RPAREN)) {
      break;
    }

    consume(TokenType::COMMA, "Expected ',' or ')' in argument list");
  }

  return args;
}

} // namespace AIParser