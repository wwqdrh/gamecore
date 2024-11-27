#pragma once

#include "collection.h"
#include "commands.h"
#include "history.h"
#include "types.h"
#include <algorithm>
#include <memory>
#include <optional>
#include <regex>
#include <string>
#include <variant>
#include <vector>

#include "lock.h"

namespace gameconsole {
class Console {
private:
  std::shared_ptr<History> _history;
  std::shared_ptr<Commands> _commands;
  std::function<void(const std::string &msg)> _out;
  // std::regex _erase_bb_tags_regex;
  std::regex _regex_command;

  mutable ReentrantRWLock rwlock;

public:
  Console(std::function<void(const std::string &msg)> out)
      : _history(std::make_shared<History>(out)),
        _commands(std::make_shared<Commands>()), _out(out) {
    // _erase_bb_tags_regex =
    //     std::regex("\\[[\\/]?[a-z0-9\\=\\#\\ \\_\\-\\,\\.\\;]+\\]");
    // 正则表达式模式:
    // ([a-zA-Z]+)    - 捕获命令名(字母组成)
    // \(             - 左括号
    // (              - 开始捕获参数组
    //   [^,)]*       - 匹配除逗号和右括号外的任意字符
    //   (?:         - 非捕获组开始
    //     ,         - 逗号
    //     [^,)]*    - 匹配除逗号和右括号外的任意字符
    //   )*          - 重复0次或多次
    // )             - 结束捕获参数组
    // \)            - 右括号
    _regex_command = std::regex(R"(([a-zA-Z]+)\(([^,)]*(?:,[^,)]*)*)\))");
  }

  // 拷贝构造函数
  Console(const Console &other) {
    auto guard = other.rwlock.shared_lock();

    // 逐个成员进行拷贝
    _history = other._history;   // 假设History类有正确的拷贝构造
    _commands = other._commands; // 假设Commands类有正确的拷贝构造
    _out = other._out;           // std::function可以直接拷贝
    _regex_command = other._regex_command; // std::regex可以直接拷贝
    // _mutex不需要拷贝,会使用默认构造函数初始化
  }

  Console &operator=(const Console &other) {
    if (this != &other) {
      // 需要加锁保护对other的读取
      auto guard = other.rwlock.shared_lock();

      _history = other._history;   // 假设History类有正确的拷贝构造
      _commands = other._commands; // 假设Commands类有正确的拷贝构造
      _out = other._out;           // std::function可以直接拷贝
      _regex_command = other._regex_command; // std::regex可以直接拷贝
    }
    return *this;
  }

public:
  void writen(const std::string &message) const {
    auto guard = rwlock.shared_lock();
    if (_out) {
      _out(message + "\n");
    }
  }
  std::optional<std::shared_ptr<Command>>
  get_command(const std::string &name) const {
    auto guard = rwlock.shared_lock();

    std::optional<Variant> val = _commands->get_value(name);
    if (val.has_value()) {
      return std::get<std::shared_ptr<Command>>(val.value());
    } else {
      return std::nullopt;
    }
  }
  std::shared_ptr<Collection> get_commands(const std::string &name) const {
    auto guard = rwlock.shared_lock();
    return _commands->find(name);
  }

  void add_command(const std::string &name,
                   const std::function<Variant(std::vector<Variant>)> &target,
                   const std::vector<std::shared_ptr<Argument>> &arguments,
                   const std::string &description) const {
    auto guard = rwlock.shared_lock();

    _commands->set_value(
        name, std::make_shared<Command>(name, target, arguments, description));
  }

  bool register_command(const std::string &name,
                        std::shared_ptr<Command> cmd) const {
    auto guard = rwlock.shared_lock();

    if (!_commands->contains_key(name)) {
      _commands->set_value(name, cmd);
      return true;
    } else {
      return false;
    }
  }

  void remove_command(const std::string &name) const {
    auto guard = rwlock.shared_lock();
    _commands->remove(name);
  }

  std::string autocomplete(const std::string &command_name) const {
    auto guard = rwlock.shared_lock();

    auto commands = _commands->find(command_name);
    if (commands->size() == 1) {
      if (commands->first().has_value()) {
        auto val =
            std::get<std::shared_ptr<Command>>(commands->first().value());
        return val->name;
      }
    }

    std::vector<std::string> all_prefix_commands = {};
    while (commands->has_next()) {
      auto val = commands->next();
      if (!val.has_value()) {
        continue;
      }

      std::string cur_name =
          std::get<std::shared_ptr<Command>>(val.value())->name;
      if (command_name == cur_name) {
        continue;
      }
      all_prefix_commands.push_back(cur_name);
    }
    if (all_prefix_commands.size() == 0) {
      return command_name;
    }

    int idx = -1;
    std::string cur_prefix = "";
    std::sort(all_prefix_commands.begin(), all_prefix_commands.end());
    while (true) {
      char cur = 0;
      idx += 1;
      for (auto item : all_prefix_commands) {
        if (idx >= item.length()) {
          return cur_prefix;
        }

        if (!cur) {
          cur = item[idx];
        } else if (cur != item[idx]) {
          return cur_prefix;
        }
      }
      cur_prefix += all_prefix_commands[0][idx];
    }
    return cur_prefix;
  }

  void execute(const std::string &input) const {
    auto guard = rwlock.shared_lock();

    writen("[color=#999999]$[/color] " + input);

    std::smatch matches;
    if (!std::regex_match(input, matches, _regex_command)) {
      return;
    }
    // 提取命令名
    std::string command = matches[1].str();
    // 提取参数字符串
    std::string params_str = matches[2].str();
    // 分割参数字符串
    std::vector<std::string> params;
    if (!params_str.empty()) {
      size_t start = 0;
      size_t end = 0;
      while ((end = params_str.find(',', start)) != std::string::npos) {
        std::string param = params_str.substr(start, end - start);
        trim(param); // 移除首尾空白
        params.push_back(param);
        start = end + 1;
      }
      // 添加最后一个参数
      std::string param = params_str.substr(start);
      trim(param);
      params.push_back(param);
    }

    auto cmd = get_command(command);

    if (cmd.has_value()) {
      cmd.value()->execute(params);
      _history->push(input);
    }
  }

private:
  void trim(std::string &s) const {
    s.erase(s.begin(), std::find_if(s.begin(), s.end(), [](unsigned char ch) {
              return !std::isspace(ch);
            }));
    s.erase(std::find_if(s.rbegin(), s.rend(),
                         [](unsigned char ch) { return !std::isspace(ch); })
                .base(),
            s.end());
  }
};
} // namespace gameconsole