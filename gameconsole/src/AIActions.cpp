#include "AIParser/AIActions.h"
// #include "godot_cpp/core/error_macros.hpp"
// #include "wrappers.h"
#include <chrono>
#include <iostream>
#include <random>
#include <variant>

namespace AIParser {

// ActionRegistry实现
// ActionRegistry &ActionRegistry::getInstance() {
//   static ActionRegistry instance;
//   return instance;
// }

void ActionRegistry::registerAction(const std::string &name, ActionFunc func,
                                    bool is_builtin) {
  // WARN_PRINT(vformat("register a action %s", godot::TO_GSTR(name)));
  actions[name] = func;
  if (is_builtin) {
    builtin_actions.insert(name);
  }
}

bool ActionRegistry::isBuiltinAction(const std::string &name) const {
  return builtin_actions.find(name) != builtin_actions.end();
}

bool ActionRegistry::hasAction(const std::string &name) const {
  return actions.find(name) != actions.end();
}

Value ActionRegistry::executeAction(const std::string &name,
                                    const std::vector<Value> &args) const {
  auto it = actions.find(name);
  if (it != actions.end()) {
    return it->second(args);
  }

  // 默认返回nullptr表示未找到
  return nullptr;
}

ActionFunc BuiltinActions::fn;

// BuiltinActions实现
void BuiltinActions::registerAll(ActionRegistry &registry) {
  registry.registerAction("randi_range", randi_range, true);
  registry.registerAction("randf_range", randf_range, true);
  registry.registerAction("chase_player", chase_player, true);
  registry.registerAction("flee", flee, true);
  registry.registerAction("patrol", patrol, true);
  registry.registerAction("attack", attack, true);
  registry.registerAction("find_heal", find_heal, true);
  registry.registerAction("play_animation", play_animation, true);
  registry.registerAction("face_player", face_player, true);
  registry.registerAction("show_dialog", show_dialog, true);
  registry.registerAction("hide_dialog", hide_dialog, true);
}

Value BuiltinActions::randi_range(const std::vector<Value> &args) {
  if (args.size() != 2) {
    return nullptr;
  }
  if (std::holds_alternative<int>(args[0]) &&
      std::holds_alternative<int>(args[1])) {
    // 使用时间种子确保每次运行不同
    static std::mt19937 gen(static_cast<unsigned>(
        std::chrono::steady_clock::now().time_since_epoch().count()));

    // 创建均匀分布
    std::uniform_int_distribution<int> dist(std::get<int>(args[0]),
                                            std::get<int>(args[1]));

    int res = dist(gen);
    return Value(res);
  }
  return nullptr;
}

Value BuiltinActions::randf_range(const std::vector<Value> &args) {
  if (args.size() != 2) {
    return nullptr;
  }
  if (std::holds_alternative<float>(args[0]) &&
      std::holds_alternative<float>(args[1])) {
    // 使用时间种子确保每次运行不同
    static std::mt19937 gen(static_cast<unsigned>(
        std::chrono::steady_clock::now().time_since_epoch().count()));

    // 创建均匀分布
    std::uniform_real_distribution<float> dist(std::get<float>(args[0]),
                                               std::get<float>(args[1]));
    // 创建均匀分布

    return dist(gen);
  }
  return nullptr;
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

} // namespace AIParser