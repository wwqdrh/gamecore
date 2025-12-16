#pragma once
#include "ASTNode.h"
#include <memory>
#include <unordered_map>

namespace AIParser {

// 行为树执行器
class BehaviorTree {
public:
  BehaviorTree();
  ~BehaviorTree();

  bool loadFromString(const std::string &expression);
  bool loadFromFile(const std::string &filename);

  // 执行行为树
  Value execute();

  // 黑板操作
  void setBlackboardValue(const std::string &key, const Value &value);
  Value getBlackboardValue(const std::string &key) const;
  bool hasBlackboardValue(const std::string &key) const;

  // 调试信息
  std::string getTreeStructure() const;
  void enableDebug(bool enabled) { debugEnabled = enabled; }

private:
  std::shared_ptr<ASTNode> root;
  std::unordered_map<std::string, Value> blackboard;
  bool debugEnabled;

  void log(const std::string &message) const;
};

} // namespace AIParser