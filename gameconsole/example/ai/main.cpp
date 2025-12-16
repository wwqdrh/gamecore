#include "AIParser.hpp"
#include <chrono>
#include <iostream>
#include <thread>

using namespace AIParser;

// 模拟游戏环境
class GameSimulation {
private:
  BehaviorTree enemyAI;
  BehaviorTree npcAI;

public:
  void setupEnemyAI() {
    enemyAI.registerAction("extra_func", [this](
                                             const std::vector<Value> &args) {
      if (std::holds_alternative<int>(args[0])) {
        std::cout << "funcindex: " << std::get<int>(args[0]) << std::endl;
      }
      // return Value("AI_END");
      if (std::holds_alternative<int>(args[1]),
          std::holds_alternative<int>(args[2])) {
        std::cout << "enemy now action extra_func: " << std::get<int>(args[1])
                  << ", " << std::get<int>(args[2]) << std::endl;
      }
      return Value(true);
    });
    enemyAI.bind_actionfn([this](const std::vector<Value> &args) {
      std::cout << "here" << std::endl;
      if (std::holds_alternative<std::string>(args[1])) {
        std::cout << "enemy now action: " << std::get<std::string>(args[0])
                  << std::endl;
      }
      return true;
    });
    std::string enemy_ai = R"(
          sequence(
            repeat(extra_func(1, 2), 3),
            extra_func(1, 2),
            selector(
                if(health < 30, sequence(flee(), find_heal())),
                if(can_see_player, 
                    selector(
                        if(distance < 50, attack()),
                        chase_player()
                    )
                ),
                patrol()
            )
          )
        )";

    if (enemyAI.loadFromString(enemy_ai)) {
      std::cout << "Enemy AI loaded successfully!\n";

      // 设置黑板值
      enemyAI.setBlackboardValue("health", 100);
      enemyAI.setBlackboardValue("can_see_player", true);
      enemyAI.setBlackboardValue("distance", 75.0f);

      enemyAI.enableDebug(true);
    } else {
      std::cout << "Failed to load enemy AI\n";
    }
  }

  void setupNPCAI() {
    npcAI.bind_actionfn([this](const std::vector<Value> &args) {
      if (std::holds_alternative<std::string>(args[0])) {
        std::cout << "enemy now action: " << std::get<std::string>(args[0])
                  << std::endl;
      }
      return true;
    });
    std::string npc_ai = R"(
            sequence(
                face_player(),
                play_animation('wave'),
                wait(0.5),
                show_dialog('Hello traveler!'),
                wait_for_input(),
                hide_dialog()
            )
        )";

    if (npcAI.loadFromString(npc_ai)) {
      std::cout << "NPC AI loaded successfully!\n";

      npcAI.enableDebug(true);
    } else {
      std::cout << "Failed to load NPC AI\n";
    }
  }

  void runEnemySimulation() {
    std::cout << "\n=== Running Enemy AI Simulation ===\n";

    // 模拟不同情况
    std::cout << "\nScenario 1: High health, can see player, far distance\n";
    enemyAI.setBlackboardValue("health", 100);
    enemyAI.setBlackboardValue("can_see_player", true);
    enemyAI.setBlackboardValue("distance", 75.0f);
    enemyAI.execute(3);

    // std::cout << "\nScenario 2: Low health\n";
    // enemyAI.setBlackboardValue("health", 20);
    // enemyAI.execute();

    // std::cout << "\nScenario 3: Cannot see player\n";
    // enemyAI.setBlackboardValue("can_see_player", false);
    // enemyAI.execute();

    // std::cout << "\nScenario 4: Close distance\n";
    // enemyAI.setBlackboardValue("can_see_player", true);
    // enemyAI.setBlackboardValue("distance", 30.0f);
    // enemyAI.execute();
  }

  void runNPCSimulation() {
    std::cout << "\n=== Running NPC AI Simulation ===\n";
    npcAI.execute();
  }

  void interactiveDemo() {
    std::cout << "\n=== Interactive AI Demo ===\n";
    std::cout << "Available commands:\n";
    std::cout << "  set <key> <value>  - Set blackboard value\n";
    std::cout << "  run                - Run AI\n";
    std::cout << "  tree               - Show behavior tree structure\n";
    std::cout << "  exit               - Exit demo\n";

    std::string command;
    while (true) {
      std::cout << "\n> ";
      std::getline(std::cin, command);

      if (command == "exit")
        break;

      if (command.substr(0, 4) == "set ") {
        // 解析设置命令
        size_t firstSpace = command.find(' ', 4);
        if (firstSpace != std::string::npos) {
          std::string key = command.substr(4, firstSpace - 4);
          std::string valueStr = command.substr(firstSpace + 1);

          // 简单类型推断
          if (valueStr == "true") {
            enemyAI.setBlackboardValue(key, true);
            std::cout << "Set " << key << " = true\n";
          } else if (valueStr == "false") {
            enemyAI.setBlackboardValue(key, false);
            std::cout << "Set " << key << " = false\n";
          } else if (valueStr.find('.') != std::string::npos) {
            try {
              float value = std::stof(valueStr);
              enemyAI.setBlackboardValue(key, value);
              std::cout << "Set " << key << " = " << value << "\n";
            } catch (...) {
              std::cout << "Invalid float value\n";
            }
          } else {
            try {
              int value = std::stoi(valueStr);
              enemyAI.setBlackboardValue(key, value);
              std::cout << "Set " << key << " = " << value << "\n";
            } catch (...) {
              enemyAI.setBlackboardValue(key, valueStr);
              std::cout << "Set " << key << " = \"" << valueStr << "\"\n";
            }
          }
        }
      } else if (command == "run") {
        enemyAI.execute();
      } else if (command == "tree") {
        std::cout << enemyAI.getTreeStructure();
      } else if (command == "help") {
        std::cout << "Available commands:\n";
        std::cout << "  set <key> <value>  - Set blackboard value\n";
        std::cout << "  run                - Run AI\n";
        std::cout << "  tree               - Show behavior tree structure\n";
        std::cout << "  exit               - Exit demo\n";
      } else {
        std::cout << "Unknown command. Type 'help' for available commands.\n";
      }
    }
  }
};

int main() {
  GameSimulation sim;

  std::cout << "=== Godot AI Parser Demo ===\n";

  // 设置AI
  sim.setupEnemyAI();
  sim.setupNPCAI();

  // 运行模拟
  sim.runEnemySimulation();

  // // 等待一下再运行NPC AI
  // std::this_thread::sleep_for(std::chrono::milliseconds(1000));
  // sim.runNPCSimulation();

  // // 交互式演示
  // sim.interactiveDemo();

  std::cout << "\nDemo completed. Goodbye!\n";

  return 0;
}