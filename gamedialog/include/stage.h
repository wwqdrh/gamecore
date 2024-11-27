#pragma once

#include "flow.h"
#include "timeline.h"
#include "word.h"
#include <memory>
#include <sstream>
#include <string>
#include <variant>
#include <vector>

namespace gamedialog {
class ControlFlow;
class DiaStage {
public:
  using LineVariant =
      std::variant<std::shared_ptr<DialogueWord>, std::shared_ptr<ControlFlow>>;

private:
  std::string stage_name;
  std::vector<std::string> stage_flags;
  std::vector<LineVariant> dialogue_keys;
  int current_ = 0;
  Timeline *timeline_;

public:
  DiaStage() = default;
  DiaStage(const std::string &data);
  DiaStage(const std::vector<std::string> &data) { initial(data); }

public:
  void initial(const std::vector<std::string> &data);
  void set_timeline(Timeline *timeline) { timeline_ = timeline; }
  std::string get_stage_name() const { return stage_name; }
  int get_line_size() const { return dialogue_keys.size(); }
  const std::vector<std::string> &get_flags() { return stage_flags; }
  bool has_next() const;
  std::shared_ptr<DialogueWord> next();
  void clean() { current_ = 0; }
  bool is_start() { return current_ == 0; }
  bool is_doing() { return current_ > 0 && current_ < dialogue_keys.size(); }

private:
  void _parse_section(const std::vector<std::string> &names,
                      const std::vector<std::string> &words);
  // 检查字符串是否为空或只包含空白字符
  bool is_empty(const std::string &str) const {
    return str.find_first_not_of(" \t\n\r") == std::string::npos;
  }
  // 分割字符串
  std::vector<std::string> split(const std::string &str, char delimiter) const {
    std::vector<std::string> tokens;
    std::string token;
    std::istringstream token_stream(str);
    while (std::getline(token_stream, token, delimiter)) {
      tokens.push_back(token);
    }
    return tokens;
  }
  std::string strip(const std::string &str) const {
    size_t first = str.find_first_not_of(" \t\n\r");
    if (first == std::string::npos)
      return "";
    size_t last = str.find_last_not_of(" \t\n\r");
    return str.substr(first, (last - first + 1));
  }
  // 辅助函数：去除前缀
  std::string trim_prefix(const std::string &str,
                          const std::string &prefix) const {
    if (starts_with(str, prefix)) {
      return str.substr(prefix.length());
    }
    return str;
  }

  // 辅助函数：去除后缀
  std::string trim_suffix(const std::string &str,
                          const std::string &suffix) const {
    if (ends_with(str, suffix)) {
      return str.substr(0, str.length() - suffix.length());
    }
    return str;
  }
  // 辅助函数：检查字符串是否以特定前缀开始
  bool starts_with(const std::string &str, const std::string &prefix) const {
    return str.size() >= prefix.size() &&
           str.compare(0, prefix.size(), prefix) == 0;
  }

  // 辅助函数：检查字符串是否以特定后缀结束
  bool ends_with(const std::string &str, const std::string &suffix) const {
    return str.size() >= suffix.size() &&
           str.compare(str.size() - suffix.size(), suffix.size(), suffix) == 0;
  }
};
} // namespace gamedialog