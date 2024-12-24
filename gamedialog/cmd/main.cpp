#include "scene_manager.h"
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

void load_variables(const std::string &filepath) {
  std::ifstream file(filepath);
  if (!file.is_open()) {
    std::cerr << "无法打开变量文件: " << filepath << std::endl;
    return;
  }

  std::string line;
  while (std::getline(file, line)) {
    size_t pos = line.find('=');
    if (pos != std::string::npos) {
      std::string key = line.substr(0, pos);
      std::string value = line.substr(pos + 1);
      gamedialog::SceneManager::instance().set_variable(key, value);
    }
  }
}

void load_timeline(const std::string &name, const std::string &filepath) {
  std::ifstream file(filepath);
  if (!file.is_open()) {
    std::cerr << "无法打开timeline文件: " << filepath << std::endl;
    return;
  }

  std::stringstream buffer;
  buffer << file.rdbuf();
  gamedialog::SceneManager::instance().add_timeline(name, buffer.str());
}

void print_help() {
  std::cout << "可用命令：\n"
            << "vars - 显示所有全局变量\n"
            << "next - 获取下一个对话内容\n"
            << "current - 显示当前timeline名称\n"
            << "switch <timeline_name> - 切换到指定timeline\n"
            << "help - 显示此帮助信息\n"
            << "quit - 退出程序\n";
}

void print_variables() {
  std::cout << "当前已设置的变量状态：" << std::endl;
  auto variables = gamedialog::SceneManager::instance().get_all_variables();
  if (variables.empty()) {
    std::cout << "  (无变量)" << std::endl;
    return;
  }

  for (const auto &var : variables) {
    std::cout << "  " << var << std::endl;
  }
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    std::cout << "用法: " << argv[0]
              << " <variables.txt> [timeline1.txt] [timeline2.txt] ...\n";
    return 1;
  }

  // 加载变量文件
  load_variables(argv[1]);

  // 加载timeline文件
  for (int i = 2; i < argc; i++) {
    std::string timeline_name = "timeline_" + std::to_string(i - 1);
    load_timeline(timeline_name, argv[i]);
    if (i == 2) {
      // 设置第一个timeline为当前timeline
      gamedialog::SceneManager::instance().set_current_timeline(timeline_name);
    }
  }

  std::string command;
  print_help();

  while (true) {
    std::cout << "> ";
    std::getline(std::cin, command);

    if (command == "quit") {
      break;
    } else if (command == "help") {
      print_help();
    } else if (command == "vars") {
      print_variables();
    } else if (command == "next") {
      auto *timeline =
          gamedialog::SceneManager::instance().get_current_timeline();
      if (timeline) {
        // TODO: 需要在Timeline类中实现获取下一个对话的方法
        std::cout << "获取下一个对话内容" << std::endl;
        std::map<std::string, std::vector<std::string>> stages =
            gamedialog::SceneManager::instance().get_all_available_stages();
        for (const auto &stage : stages) {
          std::cout << "  " << stage.first << std::endl;
          for (const auto &item : stage.second) {
            std::cout << "    " << item << std::endl;
          }
        }
      } else {
        std::cout << "当前没有活动的timeline" << std::endl;
      }
    } else if (command == "current") {
      auto *timeline =
          gamedialog::SceneManager::instance().get_current_timeline();
      if (timeline) {
        std::cout << "当前timeline: "
                  << gamedialog::SceneManager::instance().get_current_timeline()
                  << std::endl;
      } else {
        std::cout << "当前没有活动的timeline" << std::endl;
      }
    } else if (command.substr(0, 6) == "switch") {
      std::string timeline_name = command.substr(7);
      gamedialog::SceneManager::instance().set_current_timeline(timeline_name);
      std::cout << "已切换到timeline: " << timeline_name << std::endl;
    } else {
      std::cout << "未知命令。输入 'help' 取帮助。" << std::endl;
    }
  }

  return 0;
}