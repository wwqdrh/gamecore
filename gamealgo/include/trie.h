#pragma once

#include <memory>
#include <string>
#include <unordered_map>

namespace gamealgo {
class TrieNode {
public:
  std::unordered_map<char, std::unique_ptr<TrieNode>> children;
  std::string value;

  TrieNode() : value("") {}

  bool hasChild(char index) const {
    return children.find(index) != children.end();
  }

  TrieNode *getChild(char index) { return children[index].get(); }

  void initializeChildAt(char index) {
    children[index] = std::make_unique<TrieNode>();
  }
};

class Trie {
private:
  std::unique_ptr<TrieNode> root;

public:
  Trie() : root(std::make_unique<TrieNode>()) {}

  void insert(const std::string &key, const std::string &value) {
    TrieNode *currentNode = root.get();
    for (char c : key) {
      if (!currentNode->hasChild(c)) {
        currentNode->initializeChildAt(c);
      }
      currentNode = currentNode->getChild(c);
    }
    currentNode->value = value;
  }

  bool has(const std::string &key) { return get(key) != ""; }

  const std::string get(const std::string &key) {
    TrieNode *currentNode = root.get();
    for (char c : key) {
      if (!currentNode->hasChild(c)) {
        return "";
      }
      currentNode = currentNode->getChild(c);
    }
    return currentNode->value;
  }
};
} // namespace libs