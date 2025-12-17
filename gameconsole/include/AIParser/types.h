#pragma once

#include <functional>
#include <string>
#include <variant>
#include <vector>
namespace AIParser {
// 值类型
using Value = std::variant<bool, int, float, std::string, nullptr_t>;
const std::string END_FLAG = "AI_END";
using ActionFunc = std::function<Value(const std::vector<Value> &)>;
} // namespace AIParser