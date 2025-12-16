#include "AIParser/ASTNode.h"
#include "AIParser/AIActions.h"
// #include "godot_cpp/core/error_macros.hpp"
// #include "godot_cpp/variant/variant.hpp"
// #include "wrappers.h"
#include <iomanip>
#include <iostream>
#include <sstream>
#include <variant>

namespace AIParser {

// SelectorNode实现
Value SelectorNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                             int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  if (debugEnabled) {
    log("Selector: executing " + std::to_string(children.size()) + " children");
  }

  for (size_t i = 0; i < children.size(); i++) {
    if (debugEnabled) {
      log("Selector: trying child " + std::to_string(i));
    }

    Value result = children[i]->evaluate(blackboard, start_index);
    if (std::holds_alternative<std::string>(result) &&
        std::get<std::string>(result) == END_FLAG) {
      return END_FLAG;
    }

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
Value SequenceNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                             int start_index) {
  // WARN_PRINT("do sequence now");
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  if (debugEnabled) {
    log("Sequence: executing " + std::to_string(children.size()) + " children");
  }

  Value lastResult = nullptr;

  for (size_t i = 0; i < children.size(); i++) {
    if (debugEnabled) {
      log("Sequence: executing child " + std::to_string(i));
    }

    lastResult = children[i]->evaluate(blackboard, start_index);
    if (std::holds_alternative<std::string>(lastResult) &&
        std::get<std::string>(lastResult) == END_FLAG) {
      return END_FLAG;
    }

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
Value RepeatNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                           int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
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

    Value countValue = countOrCondition->evaluate(blackboard, start_index);

    if (std::holds_alternative<std::string>(countValue) &&
        std::get<std::string>(countValue) == END_FLAG) {

      return END_FLAG;
    }
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

      lastResult = child->evaluate(blackboard, start_index);
      child->treeIndex += 1;
      if (std::holds_alternative<std::string>(lastResult) &&
          std::get<std::string>(lastResult) == END_FLAG) {
        return END_FLAG;
      }

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

      lastResult = child->evaluate(blackboard, start_index);
      if (std::holds_alternative<std::string>(lastResult) &&
          std::get<std::string>(lastResult) == END_FLAG) {
        return END_FLAG;
      }

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
        Value stopCondition =
            countOrCondition->evaluate(blackboard, start_index);
        if (std::holds_alternative<std::string>(stopCondition) &&
            std::get<std::string>(stopCondition) == END_FLAG) {
          return END_FLAG;
        } else if (std::holds_alternative<bool>(stopCondition) &&
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
Value IfNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                       int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  if (debugEnabled) {
    log("If: evaluating condition");
  }

  Value conditionResult = condition->evaluate(blackboard, start_index);
  if (std::holds_alternative<std::string>(conditionResult) &&
      std::get<std::string>(conditionResult) == END_FLAG) {
    return END_FLAG;
  }

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
    return trueBranch->evaluate(blackboard, start_index);
  } else if (falseBranch) {
    if (debugEnabled) {
      log("If: executing false branch");
    }
    return falseBranch->evaluate(blackboard, start_index);
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
Value ActionNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                           int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  if (debugEnabled) {
    log("Action: executing " + name);
  }

  // 评估参数
  std::vector<Value> evaluatedArgs;
  for (const auto &arg : args) {
    auto res = arg->evaluate(blackboard, start_index);
    if (std::holds_alternative<std::string>(res) &&
        std::get<std::string>(res) == END_FLAG) {
      return END_FLAG;
    }
    evaluatedArgs.push_back(res);
  }

  // 从注册表中执行动作
  auto &registry = ActionRegistry::getInstance();
  if (registry.hasAction(name)) {
    // WARN_PRINT(
    //     godot::vformat("Action: %s found in registry",
    //     godot::TO_GSTR(name)));
    // if (debugEnabled) {
    //   log("Action: " + name + " found in registry");
    // }
    return registry.executeAction(name, evaluatedArgs);
  } else {
    // WARN_PRINT(
    //     godot::vformat("Action: %s not found in registry, returning nullptr",
    //                    godot::TO_GSTR(name)));
    // if (debugEnabled) {
    //   log("Action: " + name + " not found in registry, returning nullptr");
    // }
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
    std::unordered_map<std::string, Value> &blackboard, int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  Value leftVal = left->evaluate(blackboard, start_index);
  if (std::holds_alternative<std::string>(leftVal) &&
      std::get<std::string>(leftVal) == END_FLAG) {
    return END_FLAG;
  }
  Value rightVal = right->evaluate(blackboard, start_index);
  if (std::holds_alternative<std::string>(rightVal) &&
      std::get<std::string>(rightVal) == END_FLAG) {
    return END_FLAG;
  }
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
Value ValueNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                          int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
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
Value VariableNode::evaluate(std::unordered_map<std::string, Value> &blackboard,
                             int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
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
    std::unordered_map<std::string, Value> &blackboard, int start_index) {
  if (treeIndex != -1 && treeIndex < start_index) {
    return true;
  }
  // 对于非控制流的函数调用，当作Action处理
  // WARN_PRINT("do function now");
  // if (debugEnabled) {
  //   log("FunctionCall: " + name);
  // }

  std::vector<Value> evaluatedArgs;
  evaluatedArgs.push_back(
      treeIndex); // 新增当前function的index，用于使用者提供从某个位置恢复执行的能力
  for (const auto &arg : args) {
    auto res = arg->evaluate(blackboard, start_index);
    if (std::holds_alternative<std::string>(res) &&
        std::get<std::string>(res) == END_FLAG) {
      return END_FLAG;
    }
    evaluatedArgs.push_back(res);
  }

  auto &registry = ActionRegistry::getInstance();
  if (registry.hasAction(name)) {
    // WARN_PRINT(godot::vformat("%s is in registry", godot::TO_GSTR(name)));
    return registry.executeAction(name, evaluatedArgs);
  }
  //  else {
  // WARN_PRINT(godot::vformat("%s is not in registry", godot::TO_GSTR(name)));
  // }

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