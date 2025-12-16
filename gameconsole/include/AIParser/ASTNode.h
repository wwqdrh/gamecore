#pragma once
#include <iostream>
#include <memory>
#include <string>
#include <unordered_map>
#include <variant>
#include <vector>

namespace AIParser {

// 节点类型枚举
enum class NodeType {
  // 控制流节点
  SELECTOR,
  SEQUENCE,
  IF,
  REPEAT,

  // 行为节点
  ACTION,
  CONDITION,

  // 值节点
  BOOL,
  NUMBER,
  STRING,
  VARIABLE,

  // 操作符
  LESS,
  GREATER,
  EQUAL,
  NOT_EQUAL,
  LESS_EQUAL,
  GREATER_EQUAL,

  // 函数调用
  FUNCTION_CALL
};

// 值类型
using Value = std::variant<bool, int, float, std::string, nullptr_t>;
using ActionFunc = std::function<Value(const std::vector<Value> &)>;
const std::string END_FLAG = "AI_END";

// AST节点基类
class ASTNode {
public:
  ASTNode(NodeType type) : type(type), debugEnabled(false), treeIndex(-1) {}
  virtual ~ASTNode() = default;

  NodeType getType() const { return type; }
  virtual Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                         int start_index = -1) = 0;
  virtual std::string toString(int indent = 0) const = 0;

  // 添加调试控制
  void setDebug(bool enabled) { debugEnabled = enabled; }
  bool getDebug() const { return debugEnabled; }
  void log(const std::string &message) const {
    if (debugEnabled) {
      std::cout << "[AI Debug] " << message << std::endl;
    }
  }

protected:
  NodeType type;

public:
  bool debugEnabled;
  int treeIndex;
};

// 控制流节点
class ControlNode : public ASTNode {
public:
  ControlNode(NodeType type, std::vector<std::shared_ptr<ASTNode>> children)
      : ASTNode(type), children(children) {}

  std::vector<std::shared_ptr<ASTNode>> children;
};

class SelectorNode : public ControlNode {
public:
  SelectorNode(std::vector<std::shared_ptr<ASTNode>> children)
      : ControlNode(NodeType::SELECTOR, children) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;
};

class SequenceNode : public ControlNode {
public:
  SequenceNode(std::vector<std::shared_ptr<ASTNode>> children)
      : ControlNode(NodeType::SEQUENCE, children) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;
};

// 在ControlNode相关声明后添加
// 基本用法：重复3次
// var basic_repeat = "repeat(patrol(), 3)"
// // 重复直到成功
// var until_success = """
// repeat(
//     try_open_door(),
//     while(door_locked == true)
// )
// """
// // 嵌套使用
// var complex_ai = """
// sequence(
//     patrol(),
//     repeat(scan_area(), 5),
//     if(enemy_spotted,
//         repeat(attack(), until_dead()),
//         rest()
//     )
// )
// """
class RepeatNode : public ControlNode {
public:
  enum class RepeatMode {
    COUNT,        // 重复指定次数
    UNTIL_SUCCESS // 直到成功为止
  };

  RepeatNode(std::shared_ptr<ASTNode> child,
             std::shared_ptr<ASTNode> countOrCondition = nullptr,
             RepeatMode mode = RepeatMode::COUNT)
      : ControlNode(NodeType::REPEAT, {child}),
        countOrCondition(countOrCondition), mode(mode) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

public:
  RepeatMode mode;
  std::shared_ptr<ASTNode> countOrCondition;
};

class IfNode : public ASTNode {
public:
  IfNode(std::shared_ptr<ASTNode> condition,
         std::shared_ptr<ASTNode> trueBranch,
         std::shared_ptr<ASTNode> falseBranch = nullptr)
      : ASTNode(NodeType::IF), condition(condition), trueBranch(trueBranch),
        falseBranch(falseBranch) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

private:
  std::shared_ptr<ASTNode> condition;
  std::shared_ptr<ASTNode> trueBranch;
  std::shared_ptr<ASTNode> falseBranch;
};

// 行为节点
class ActionNode : public ASTNode {
public:
  ActionNode(const std::string &name,
             std::vector<std::shared_ptr<ASTNode>> args = {})
      : ASTNode(NodeType::ACTION), name(name), args(args) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

private:
  std::string name;
  std::vector<std::shared_ptr<ASTNode>> args;
};

struct CompareVisitor {
  NodeType op;

  // 处理nullptr的情况
  Value operator()(std::nullptr_t, std::nullptr_t) const {
    switch (op) {
    case NodeType::EQUAL:
      return true;
    case NodeType::NOT_EQUAL:
      return false;
    default:
      return false;
    }
  }

  template <typename T> Value operator()(std::nullptr_t, T &&) const {
    switch (op) {
    case NodeType::EQUAL:
      return false;
    case NodeType::NOT_EQUAL:
      return true;
    default:
      return false;
    }
  }

  template <typename T> Value operator()(T &&, std::nullptr_t) const {
    switch (op) {
    case NodeType::EQUAL:
      return false;
    case NodeType::NOT_EQUAL:
      return true;
    default:
      return false;
    }
  }

  // 处理同类型比较（非nullptr）
  template <typename T> Value operator()(T &&l, T &&r) const {
    switch (op) {
    case NodeType::LESS:
      return l < r;
    case NodeType::GREATER:
      return l > r;
    case NodeType::EQUAL:
      return l == r;
    case NodeType::NOT_EQUAL:
      return l != r;
    case NodeType::LESS_EQUAL:
      return l <= r;
    case NodeType::GREATER_EQUAL:
      return l >= r;
    default:
      return false;
    }
  }

  // 处理不同类型（非nullptr）- 返回false
  template <typename T, typename U> Value operator()(T &&, U &&) const {
    return false;
  }
};

// 条件节点
class ConditionNode : public ASTNode {
public:
  ConditionNode(std::shared_ptr<ASTNode> left, NodeType op,
                std::shared_ptr<ASTNode> right)
      : ASTNode(op), left(left), right(right) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

private:
  std::shared_ptr<ASTNode> left;
  std::shared_ptr<ASTNode> right;
};

// 值节点
class ValueNode : public ASTNode {
public:
  ValueNode(const Value &value, NodeType type = NodeType::NUMBER)
      : ASTNode(type), value(value) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

public:
  Value value;
};

// 变量节点
class VariableNode : public ASTNode {
public:
  VariableNode(const std::string &name)
      : ASTNode(NodeType::VARIABLE), name(name) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

private:
  std::string name;
};

// 函数调用节点
class FunctionCallNode : public ASTNode {
public:
  FunctionCallNode(const std::string &name,
                   std::vector<std::shared_ptr<ASTNode>> args = {})
      : ASTNode(NodeType::FUNCTION_CALL), name(name), args(args) {}

  Value evaluate(std::unordered_map<std::string, Value> &blackboard,
                 int start_index = -1) override;
  std::string toString(int indent = 0) const override;

private:
  std::string name;
  std::vector<std::shared_ptr<ASTNode>> args;
};

} // namespace AIParser