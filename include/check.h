#pragma once

#include "traits.h"
#include <algorithm>
#include <cctype>
#include <functional>
#include <map>
#include <memory>
#include <regex>
#include <sstream>
#include <string>
#include <variant>
#include <vector>

namespace gamedb {

class ConditionParser {
public:
  struct ParseState {
    size_t cursor = 0;
  };

  static std::string trim(const std::string &str) {
    size_t first = str.find_first_not_of(" \t\n\r");
    if (first == std::string::npos)
      return "";
    size_t last = str.find_last_not_of(" \t\n\r");
    return str.substr(first, last - first + 1);
  }

public:
  bool checkCondition(const variantDict &property,
                      const std::string &condition) const {
    auto conditions = parseCondition(condition);
    return checkParsedConditions(property, conditions);
  }

private:
  std::vector<std::string> parseCondition(const std::string &condition) const {
    std::vector<std::string> result;
    std::string current;

    for (size_t i = 0; i < condition.length(); ++i) {
      char c = condition[i];

      switch (c) {
      case ' ': // 跳过空格
        continue;

      case '(': // 处理左括号
        if (!current.empty()) {
          result.push_back(current);
          current.clear();
        }
        // result.push_back("(");
        break;

      case ')': // 处理右括号
        if (!current.empty()) {
          result.push_back(current);
          current.clear();
        }
        // result.push_back(")");
        break;

      case '&': // 处理与运算符
      case '|': // 处理或运算符
        if (!current.empty()) {
          result.push_back(current);
          current.clear();
        }
        result.push_back(std::string(1, c));
        break;

      default: // 收集其他字符
        current += c;
        break;
      }
    }

    // 处理最后剩余的字符串
    if (!current.empty()) {
      result.push_back(current);
    }

    return result;
  }

  bool checkParsedConditions(const variantDict &property,
                             const std::vector<std::string> &conditions) const {
    if (conditions.size() == 0) {
      return true;
    } else if (conditions.size() == 1) {
      return checkProp(property, conditions[0]);
    }

    bool ret = checkParsedConditions(property, {conditions[0]});
    for (size_t i = 1; i < conditions.size(); i += 2) {
      const std::string &op = conditions[i];
      if (op == "&") {
        if (ret)
          ret = checkParsedConditions(property, {conditions[i + 1]});
      } else if (op == "|") {
        if (ret)
          return true;
        ret = checkParsedConditions(property, {conditions[i + 1]});
      } else
        return false;
    }
    return ret;
  }

  bool checkProp(const variantDict &property,
                 const std::string &condition) const {
    std::regex symbolRegex("[><!=?]");
    std::smatch match;
    if (!std::regex_search(condition, match, symbolRegex)) {
      return false;
    }

    size_t i = match.position(0);
    std::string prop = condition.substr(0, i);
    std::string symbol = condition.substr(i, condition[i + 1] == '=' ? 2 : 1);
    std::string value = condition.substr(i + symbol.length());

    auto it = property.find(prop);
    if (it == property.end())
      return false;

    variant propData = it->second;
    variant conditionData;

    if (!value.empty() && value[0] == '[') {
      // Parse array
      std::vector<std::string> array;
      std::string item;
      std::stringstream ss(value.substr(1, value.length() - 2));
      while (std::getline(ss, item, ',')) {
        array.push_back(trim(item));
      }
      conditionData = array;
    } else {
      conditionData = value;
    }

    if (symbol == ">")
      return variantToDouble(propData) > variantToDouble(conditionData);
    if (symbol == "<")
      return variantToDouble(propData) < variantToDouble(conditionData);
    if (symbol == ">=")
      return variantToDouble(propData) >= variantToDouble(conditionData);
    if (symbol == "<=")
      return variantToDouble(propData) <= variantToDouble(conditionData);

    if (symbol == "=") {
      // 处理vector类型
      if (std::holds_alternative<std::vector<std::string>>(propData) ||
          std::holds_alternative<std::vector<int>>(propData) ||
          std::holds_alternative<std::vector<double>>(propData)) {
        return isInVector(conditionData, propData);
      }
      // 可能是数字类型
      if (variantToDouble(propData) == variantToDouble(conditionData)) {
        return true;
      }
      // 非数字类型，比较类型和值是否相同
      return compareValues(propData, conditionData);
    }

    if (symbol == "?") {
      // 处理vector类型的propData
      if (std::holds_alternative<std::vector<std::string>>(propData) ||
          std::holds_alternative<std::vector<int>>(propData) ||
          std::holds_alternative<std::vector<double>>(propData)) {
        return std::visit(
            [&conditionData](const auto &vec) -> bool {
              using VecType = std::decay_t<decltype(vec)>;
              if constexpr (std::is_same_v<VecType, std::vector<std::string>> ||
                            std::is_same_v<VecType, std::vector<int>> ||
                            std::is_same_v<VecType, std::vector<double>>) {
                for (const auto &item : vec) {
                  if (isInVector(item, conditionData)) {
                    return true;
                  }
                }
              }
              return false;
            },
            propData);
      }
      // 处理非vector类型的propData
      return isInVector(propData, conditionData);
    }

    return false;
  }
};
} // namespace gamedb