#include "AIParser/BehaviorTree.h"
#include "AIParser/AIActions.h"
#include "AIParser/Parser.h"
// #include "godot_cpp/variant/variant.hpp"
#include <fstream>
#include <iostream>
#include <sstream>

namespace AIParser {

BehaviorTree::BehaviorTree() : debugEnabled(false) {
  // 注册内置行为
  BuiltinActions::registerAll();
}

BehaviorTree::~BehaviorTree() {}

bool BehaviorTree::loadFromString(const std::string &expression) {
  Parser parser(expression);
  root = parser.parse();
  return true;
}

bool BehaviorTree::loadFromFile(const std::string &filename) {
  std::ifstream file(filename);
  if (!file.is_open()) {
    std::cerr << "Failed to open file: " << filename << std::endl;
    return false;
  }

  std::stringstream buffer;
  buffer << file.rdbuf();

  return loadFromString(buffer.str());
}

Value BehaviorTree::execute(int start_index) {
  if (!root) {
    // WARN_PRINT("Behavior tree not loaded");
    std::cerr << "Behavior tree not loaded" << std::endl;
    return nullptr;
  }

  if (debugEnabled) {
    // WARN_PRINT("Executing behavior tree...");
    std::cout << "Executing behavior tree..." << std::endl;
    std::cout << getTreeStructure() << std::endl;
  }

  // root->setDebug(debugEnabled);
  // WARN_PRINT("execute here");
  Value result = root->evaluate(blackboard, start_index);

  if (debugEnabled) {
    // WARN_PRINT("Execution completed");
    std::cout << "Execution completed" << std::endl;
  }

  return result;
}

void BehaviorTree::setBlackboardValue(const std::string &key,
                                      const Value &value) {
  blackboard[key] = value;
}

Value BehaviorTree::getBlackboardValue(const std::string &key) const {
  auto it = blackboard.find(key);
  if (it != blackboard.end()) {
    return it->second;
  }
  return nullptr;
}

bool BehaviorTree::hasBlackboardValue(const std::string &key) const {
  return blackboard.find(key) != blackboard.end();
}

std::string BehaviorTree::getTreeStructure() const {
  if (!root) {
    return "Behavior tree not loaded";
  }
  return root->toString();
}

void BehaviorTree::log(const std::string &message) const {
  if (debugEnabled) {
    std::cout << "[BehaviorTree] " << message << std::endl;
  }
}

} // namespace AIParser