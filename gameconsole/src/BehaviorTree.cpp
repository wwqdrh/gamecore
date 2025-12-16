#include "AIParser/BehaviorTree.h"
#include "AIParser/AIActions.h"
#include "AIParser/Parser.h"
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
  try {
    Parser parser(expression);
    root = parser.parse();
    return true;
  } catch (const std::exception &e) {
    std::cerr << "Failed to parse behavior tree: " << e.what() << std::endl;
    return false;
  }
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

Value BehaviorTree::execute() {
  if (!root) {
    std::cerr << "Behavior tree not loaded" << std::endl;
    return nullptr;
  }

  if (debugEnabled) {
    std::cout << "Executing behavior tree..." << std::endl;
    std::cout << getTreeStructure() << std::endl;
  }

  try {
    // root->setDebug(debugEnabled);
    Value result = root->evaluate(blackboard);

    if (debugEnabled) {
      std::cout << "Execution completed" << std::endl;
    }

    return result;
  } catch (const std::exception &e) {
    std::cerr << "Error executing behavior tree: " << e.what() << std::endl;
    return nullptr;
  }
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