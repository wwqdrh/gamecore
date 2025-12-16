#include "AIParser/ASTNode.h"
#include "AIParser/AIActions.h"
#include <iomanip>
#include <iostream>
#include <sstream>

namespace AIParser {

// SelectorNode实现
Value SelectorNode::evaluate(
    std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    log("Selector: executing " + std::to_string(children.size()) + " children");
  }

  for (size_t i = 0; i < children.size(); i++) {
    if (debugEnabled) {
      log("Selector: trying child " + std::to_string(i));
    }

    Value result = children[i]->evaluate(blackboard);

    // 如果子节点执行成功，返回成功
    if (!std::holds_alternative<nullptr_t>(result)) {
      if (debugEnabled) {
        log("Selector: child " + std::to_string(i) + " succeeded");
      }
      return result;
    }
  }

  if (debugEnabled) {
    log("Selector: all children failed");
  }
  return nullptr;
}

std::string SelectorNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << "Selector:\n";
  for (const auto &child : children) {
    ss << child->toString(indent + 2);
  }

  return ss.str();
}

// SequenceNode实现
Value SequenceNode::evaluate(
    std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    log("Sequence: executing " + std::to_string(children.size()) + " children");
  }

  Value lastResult = nullptr;

  for (size_t i = 0; i < children.size(); i++) {
    if (debugEnabled) {
      log("Sequence: executing child " + std::to_string(i));
    }

    lastResult = children[i]->evaluate(blackboard);

    // 如果子节点失败，返回失败
    if (std::holds_alternative<nullptr_t>(lastResult)) {
      if (debugEnabled) {
        log("Sequence: child " + std::to_string(i) + " failed, aborting");
      }
      return nullptr;
    }
  }

  if (debugEnabled) {
    log("Sequence: all children completed successfully");
  }
  return lastResult;
}

std::string SequenceNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << "Sequence:\n";
  for (const auto &child : children) {
    ss << child->toString(indent + 2);
  }

  return ss.str();
}
// RepeatNode实现
Value RepeatNode::evaluate(std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    log("Repeat: mode = " +
        std::string(mode == RepeatMode::COUNT ? "COUNT" : "UNTIL_SUCCESS"));
  }

  if (children.empty()) {
    if (debugEnabled) {
      log("Repeat: no child to execute");
    }
    return nullptr;
  }

  auto &child = children[0];
  Value lastResult = nullptr;

  if (mode == RepeatMode::COUNT) {
    // 计数模式
    if (!countOrCondition) {
      if (debugEnabled) {
        log("Repeat: count mode but no count specified");
      }
      return nullptr;
    }

    Value countValue = countOrCondition->evaluate(blackboard);
    int repeatCount = 0;

    if (std::holds_alternative<int>(countValue)) {
      repeatCount = std::get<int>(countValue);
    } else if (std::holds_alternative<float>(countValue)) {
      repeatCount = static_cast<int>(std::get<float>(countValue));
    } else {
      if (debugEnabled) {
        log("Repeat: count is not a number");
      }
      return nullptr;
    }

    if (repeatCount <= 0) {
      if (debugEnabled) {
        log("Repeat: count <= 0, nothing to repeat");
      }
      return nullptr;
    }

    if (debugEnabled) {
      log("Repeat: executing " + std::to_string(repeatCount) + " times");
    }

    for (int i = 0; i < repeatCount; i++) {
      if (debugEnabled) {
        log("Repeat: iteration " + std::to_string(i + 1) + "/" +
            std::to_string(repeatCount));
      }

      lastResult = child->evaluate(blackboard);

      if (std::holds_alternative<nullptr_t>(lastResult)) {
        if (debugEnabled) {
          log("Repeat: failed at iteration " + std::to_string(i + 1));
        }
        return nullptr;
      }
    }

  } else {
    // UNTIL_SUCCESS模式
    if (debugEnabled) {
      log("Repeat: repeating until success");
    }

    int maxAttempts = 1000; // 安全限制，防止无限循环
    int attempts = 0;

    while (attempts < maxAttempts) {
      attempts++;
      if (debugEnabled) {
        log("Repeat: attempt " + std::to_string(attempts));
      }

      lastResult = child->evaluate(blackboard);

      // 如果成功，返回成功
      if (!std::holds_alternative<nullptr_t>(lastResult)) {
        if (debugEnabled) {
          log("Repeat: succeeded after " + std::to_string(attempts) +
              " attempts");
        }
        return lastResult;
      }

      // 可选：检查停止条件
      if (countOrCondition) {
        Value stopCondition = countOrCondition->evaluate(blackboard);
        if (std::holds_alternative<bool>(stopCondition) &&
            std::get<bool>(stopCondition)) {
          if (debugEnabled) {
            log("Repeat: stop condition met after " + std::to_string(attempts) +
                " attempts");
          }
          break;
        }
      }
    }

    if (attempts >= maxAttempts) {
      if (debugEnabled) {
        log("Repeat: reached maximum attempts (" + std::to_string(maxAttempts) +
            ")");
      }
    }

    return nullptr; // 从未成功
  }

  return lastResult;
}

std::string RepeatNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  // 根据模式选择不同的显示格式
  if (mode == RepeatMode::COUNT) {
    ss << indentStr << "Repeat(";
    if (countOrCondition) {
      ss << countOrCondition->toString(0);
    } else {
      ss << "<no count>";
    }
    ss << " times):\n";
  } else {
    ss << indentStr << "RepeatUntilSuccess(";
    if (countOrCondition) {
      ss << "while: " << countOrCondition->toString(0);
    }
    ss << "):\n";
  }

  // 显示子节点
  if (!children.empty()) {
    ss << children[0]->toString(indent + 2);
  }

  return ss.str();
}

// IfNode实现
Value IfNode::evaluate(std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    log("If: evaluating condition");
  }

  Value conditionResult = condition->evaluate(blackboard);

  // 确保条件结果是布尔值
  if (!std::holds_alternative<bool>(conditionResult)) {
    if (debugEnabled) {
      log("If: condition did not evaluate to boolean");
    }
    return nullptr;
  }

  bool cond = std::get<bool>(conditionResult);

  if (debugEnabled) {
    log("If: condition is " + std::string(cond ? "true" : "false"));
  }

  if (cond) {
    if (debugEnabled) {
      log("If: executing true branch");
    }
    return trueBranch->evaluate(blackboard);
  } else if (falseBranch) {
    if (debugEnabled) {
      log("If: executing false branch");
    }
    return falseBranch->evaluate(blackboard);
  }

  return nullptr;
}

std::string IfNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << "If:\n";
  ss << indentStr << "  Condition:\n";
  ss << condition->toString(indent + 4);
  ss << indentStr << "  TrueBranch:\n";
  ss << trueBranch->toString(indent + 4);
  if (falseBranch) {
    ss << indentStr << "  FalseBranch:\n";
    ss << falseBranch->toString(indent + 4);
  }

  return ss.str();
}

// ActionNode实现
Value ActionNode::evaluate(std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    log("Action: executing " + name);
  }

  // 评估参数
  std::vector<Value> evaluatedArgs;
  for (const auto &arg : args) {
    evaluatedArgs.push_back(arg->evaluate(blackboard));
  }

  // 从注册表中执行动作
  auto &registry = ActionRegistry::getInstance();
  if (registry.hasAction(name)) {
    if (debugEnabled) {
      log("Action: " + name + " found in registry");
    }
    return registry.executeAction(name, evaluatedArgs);
  } else {
    if (debugEnabled) {
      log("Action: " + name + " not found in registry, returning nullptr");
    }
    return nullptr;
  }
}

std::string ActionNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << "Action: " << name;
  if (!args.empty()) {
    ss << " (";
    for (size_t i = 0; i < args.size(); i++) {
      ss << args[i]->toString(0);
      if (i < args.size() - 1)
        ss << ", ";
    }
    ss << ")";
  }
  ss << "\n";

  return ss.str();
}

// ConditionNode实现
Value ConditionNode::evaluate(
    std::unordered_map<std::string, Value> &blackboard) {
  Value leftVal = left->evaluate(blackboard);
  Value rightVal = right->evaluate(blackboard);
  // 创建比较访问器

  return std::visit(CompareVisitor{type}, leftVal, rightVal);
}

std::string ConditionNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');
  std::string opStr;

  switch (type) {
  case NodeType::LESS:
    opStr = "<";
    break;
  case NodeType::GREATER:
    opStr = ">";
    break;
  case NodeType::EQUAL:
    opStr = "==";
    break;
  case NodeType::NOT_EQUAL:
    opStr = "!=";
    break;
  case NodeType::LESS_EQUAL:
    opStr = "<=";
    break;
  case NodeType::GREATER_EQUAL:
    opStr = ">=";
    break;
  default:
    opStr = "??";
  }

  ss << indentStr << "Condition: " << left->toString(0) << " " << opStr << " "
     << right->toString(0) << "\n";

  return ss.str();
}

// ValueNode实现
Value ValueNode::evaluate(std::unordered_map<std::string, Value> &blackboard) {
  if (debugEnabled) {
    std::stringstream ss;
    ss << "Value: returning ";
    std::visit(
        [&ss](auto &&arg) {
          using T = std::decay_t<decltype(arg)>;
          if constexpr (std::is_same_v<T, bool>) {
            ss << (arg ? "true" : "false");
          } else if constexpr (std::is_same_v<T, int>) {
            ss << arg;
          } else if constexpr (std::is_same_v<T, float>) {
            ss << arg;
          } else if constexpr (std::is_same_v<T, std::string>) {
            ss << "\"" << arg << "\"";
          }
        },
        value);
    log(ss.str());
  }
  return value;
}

std::string ValueNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr;
  std::visit(
      [&ss](auto &&arg) {
        using T = std::decay_t<decltype(arg)>;
        if constexpr (std::is_same_v<T, bool>) {
          ss << (arg ? "true" : "false");
        } else if constexpr (std::is_same_v<T, int>) {
          ss << arg;
        } else if constexpr (std::is_same_v<T, float>) {
          ss << std::fixed << std::setprecision(2) << arg;
        } else if constexpr (std::is_same_v<T, std::string>) {
          ss << "\"" << arg << "\"";
        } else if constexpr (std::is_same_v<T, nullptr_t>) {
          ss << "null";
        }
      },
      value);

  return ss.str();
}

// VariableNode实现
Value VariableNode::evaluate(
    std::unordered_map<std::string, Value> &blackboard) {
  auto it = blackboard.find(name);
  if (it != blackboard.end()) {
    if (debugEnabled) {
      log("Variable: " + name + " found in blackboard");
    }
    return it->second;
  }

  if (debugEnabled) {
    log("Variable: " + name + " not found in blackboard, returning nullptr");
  }
  return nullptr;
}

std::string VariableNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << name;

  return ss.str();
}

// FunctionCallNode实现
Value FunctionCallNode::evaluate(
    std::unordered_map<std::string, Value> &blackboard) {
  // 对于非控制流的函数调用，当作Action处理
  if (debugEnabled) {
    log("FunctionCall: " + name);
  }

  std::vector<Value> evaluatedArgs;
  for (const auto &arg : args) {
    evaluatedArgs.push_back(arg->evaluate(blackboard));
  }

  auto &registry = ActionRegistry::getInstance();
  if (registry.hasAction(name)) {
    return registry.executeAction(name, evaluatedArgs);
  }

  return nullptr;
}

std::string FunctionCallNode::toString(int indent) const {
  std::stringstream ss;
  std::string indentStr(indent, ' ');

  ss << indentStr << name << "(";
  for (size_t i = 0; i < args.size(); i++) {
    ss << args[i]->toString(0);
    if (i < args.size() - 1)
      ss << ", ";
  }
  ss << ")\n";

  return ss.str();
}

} // namespace AIParser