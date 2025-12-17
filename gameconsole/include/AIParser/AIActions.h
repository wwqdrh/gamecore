#pragma once
#include <functional>
#include <unordered_map>
#include <unordered_set>

#include "AIParser/types.h"

namespace AIParser {

// 行为注册器
class ActionRegistry {
public:
  // static ActionRegistry &getInstance();

  void registerAction(const std::string &name, ActionFunc func,
                      bool is_builtin = false);
  bool isBuiltinAction(const std::string &name) const;
  bool hasAction(const std::string &name) const;
  Value executeAction(const std::string &name, const std::vector<Value> &args) const;

private:
  // ActionRegistry() = default;
  std::unordered_map<std::string, ActionFunc> actions;
  std::unordered_set<std::string> builtin_actions;
};

// 内置行为实现
class BuiltinActions {
public:
  static ActionFunc fn;

public:
  static void bind_actionfn(ActionFunc fn_) { fn = fn_; }
  static void registerAll(ActionRegistry &registry);
  static std::vector<Value> copy_args(const std::string &action,
                                      const std::vector<Value> &args) {
    std::vector<Value> result;
    result.reserve(args.size() + 1); // 预分配空间以提高效率
    // 先插入新字符串
    result.push_back(action);
    result.insert(result.end(), args.begin(), args.end());
    return result;
  }

  // 数学相关
  static Value randi_range(const std::vector<Value> &args);
  static Value randf_range(const std::vector<Value> &args);
  // 移动相关
  static Value chase_player(const std::vector<Value> &args);
  static Value flee(const std::vector<Value> &args);
  static Value patrol(const std::vector<Value> &args);

  // 战斗相关
  static Value attack(const std::vector<Value> &args);
  static Value find_heal(const std::vector<Value> &args);

  // 动画相关
  static Value play_animation(const std::vector<Value> &args);
  static Value face_player(const std::vector<Value> &args);

  // 对话相关
  static Value show_dialog(const std::vector<Value> &args);
  static Value hide_dialog(const std::vector<Value> &args);

private:
  BuiltinActions() = default;
};

} // namespace AIParser