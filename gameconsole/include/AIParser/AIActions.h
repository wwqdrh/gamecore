#pragma once
#include "ASTNode.h"
#include <functional>
#include <unordered_map>

namespace AIParser {

// 行为注册器
class ActionRegistry {
public:
  using ActionFunc = std::function<Value(const std::vector<Value> &)>;

  static ActionRegistry &getInstance();

  void registerAction(const std::string &name, ActionFunc func);
  bool hasAction(const std::string &name) const;
  Value executeAction(const std::string &name, const std::vector<Value> &args);

private:
  ActionRegistry() = default;
  std::unordered_map<std::string, ActionFunc> actions;
};

// 内置行为实现
class BuiltinActions {
public:
  static actionfn fn;

public:
  static void bind_actionfn(actionfn fn_) { fn = fn_; }
  static void registerAll();

  // 移动相关
  static Value move_to(const std::vector<Value> &args);
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

  // 工具函数
  static Value wait(const std::vector<Value> &args);
  static Value wait_for_input(const std::vector<Value> &args);

private:
  BuiltinActions() = default;
};

} // namespace AIParser