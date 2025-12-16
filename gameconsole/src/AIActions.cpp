#include "AIParser/AIActions.h"
// #include "godot_cpp/core/error_macros.hpp"
// #include "wrappers.h"
#include <iostream>

namespace AIParser {

// ActionRegistry实现
ActionRegistry &ActionRegistry::getInstance() {
  static ActionRegistry instance;
  return instance;
}

void ActionRegistry::registerAction(const std::string &name, ActionFunc func) {
  // WARN_PRINT(vformat("register a action %s", godot::TO_GSTR(name)));
  actions[name] = func;
}

bool ActionRegistry::hasAction(const std::string &name) const {
  return actions.find(name) != actions.end();
}

Value ActionRegistry::executeAction(const std::string &name,
                                    const std::vector<Value> &args) {
  auto it = actions.find(name);
  if (it != actions.end()) {
    return it->second(args);
  }

  // 默认返回nullptr表示未找到
  return nullptr;
}

ActionFunc BuiltinActions::fn;

// BuiltinActions实现
void BuiltinActions::registerAll() {
  auto &registry = ActionRegistry::getInstance();

  registry.registerAction("chase_player", chase_player);
  registry.registerAction("flee", flee);
  registry.registerAction("patrol", patrol);
  registry.registerAction("attack", attack);
  registry.registerAction("find_heal", find_heal);
  registry.registerAction("play_animation", play_animation);
  registry.registerAction("face_player", face_player);
  registry.registerAction("show_dialog", show_dialog);
  registry.registerAction("hide_dialog", hide_dialog);
  registry.registerAction("wait", wait);
  registry.registerAction("wait_for_input", wait_for_input);
}

Value BuiltinActions::chase_player(const std::vector<Value> &args) {
  // std::cout << "[AI] Executing chase_player" << std::endl;
  if (fn != nullptr) {
    return fn(copy_args("chase_player", args));
  }
  return nullptr;
}

Value BuiltinActions::flee(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("flee", args));
  }
  // std::cout << "[AI] Executing flee" << std::endl;
  return nullptr;
}

Value BuiltinActions::patrol(const std::vector<Value> &args) {
  // std::cout << "[AI] Executing patrol" << std::endl;
  if (fn != nullptr) {
    return fn(copy_args("patrol", args));
  }
  return nullptr;
}

Value BuiltinActions::attack(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("attack", args));
  }
  // std::cout << "[AI] Executing attack" << std::endl;
  return nullptr;
}

Value BuiltinActions::find_heal(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("find_heal", args));
  }
  // std::cout << "[AI] Executing find_heal" << std::endl;
  return nullptr;
}

Value BuiltinActions::play_animation(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("play_animation", args));
  }
  // std::cout << "[AI] Executing play_animation" << std::endl;
  // if (!args.empty()) {
  //   std::visit(
  //       [](auto &&arg) {
  //         using T = std::decay_t<decltype(arg)>;
  //         if constexpr (std::is_same_v<T, std::string>) {
  //           std::cout << "  Animation: " << arg << std::endl;
  //         }
  //       },
  //       args[0]);
  // }
  return nullptr;
}

Value BuiltinActions::face_player(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("face_player", args));
  }
  // std::cout << "[AI] Executing face_player" << std::endl;
  return nullptr;
}

Value BuiltinActions::show_dialog(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("show_dialog", args));
  }
  // std::cout << "[AI] Executing show_dialog" << std::endl;
  // if (!args.empty()) {
  //   std::visit(
  //       [](auto &&arg) {
  //         using T = std::decay_t<decltype(arg)>;
  //         if constexpr (std::is_same_v<T, std::string>) {
  //           std::cout << "  Dialog: " << arg << std::endl;
  //         }
  //       },
  //       args[0]);
  // }
  return nullptr;
}

Value BuiltinActions::hide_dialog(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("hide_dialog", args));
  }
  // std::cout << "[AI] Executing hide_dialog" << std::endl;
  return nullptr;
}

Value BuiltinActions::wait(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("wait", args));
  }
  // std::cout << "[AI] Executing wait" << std::endl;
  // if (!args.empty()) {
  //   std::visit(
  //       [](auto &&arg) {
  //         using T = std::decay_t<decltype(arg)>;
  //         if constexpr (std::is_same_v<T, float>) {
  //           std::cout << "  Duration: " << arg << " seconds" << std::endl;
  //         } else if constexpr (std::is_same_v<T, int>) {
  //           std::cout << "  Duration: " << arg << " seconds" << std::endl;
  //         }
  //       },
  //       args[0]);
  // }
  return nullptr;
}

Value BuiltinActions::wait_for_input(const std::vector<Value> &args) {
  if (fn != nullptr) {
    return fn(copy_args("wait_for_input", args));
  }
  // std::cout << "[AI] Executing wait_for_input" << std::endl;
  return nullptr;
}

} // namespace AIParser