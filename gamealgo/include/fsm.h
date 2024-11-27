#pragma once

#include <sstream>
#include <map>
#include <string>
#include <vector>

namespace gamealgo {
// finate state machine实现
class FSM {
protected:
  std::string current_state_ = "";
  // 一个邻接表，记录每个状态的转移
  std::map<std::string, std::vector<std::string>> state_transition_ = {};

public:
  FSM() {}
  ~FSM() {}

  // 根据json字符串解析状态之间的邻接关系 {"statea": ["stateb", "statec"],
  // [stateb]stateb,statea
  void initial(const std::string &state, const std::string &statestr) {
    // 将state_transition_重置
    state_transition_.clear();

    current_state_ = state;
    state_transition_[state] = {};
    // 从类似下面的字符串解析，一行一行读取，并设置转移
    // 每一行首先将首尾空白符删掉，然后读取[]中的，剩下的使用逗号分割
    // [statea]stateb,statec
    // [stateb]stateb,statea

    std::string line;
    std::stringstream ss(statestr);
    while (getline(ss, line)) {
      line.erase(line.find_last_not_of(" \n\r\t") + 1);
      line.erase(0, line.find_first_not_of(" \n\r\t"));
      if (line.size() == 0) {
        continue;
      }

      // 寻找]下标，将[]中内容提取出来
      std::string::size_type pos = line.find(']');
      if (pos == std::string::npos) {
        continue;
      }
      std::string state = line.substr(1, pos - 1);
      line.erase(0, pos + 1);

      // 使用逗号分割
      std::string::size_type pos1 = 0;
      std::string::size_type curpos = 0;
      while ((curpos = line.find(',', pos1)) != std::string::npos) {
        state_transition_[state].push_back(line.substr(pos1, curpos - pos1));
        pos1 = curpos + 1;
      }
    }
  }
  std::string current_state() { return current_state_; }
  // 根据当前状态，执行转移，判断是否能够转移到给定的状态
  bool travel(const std::string &state) {
    if (state_transition_.find(current_state_) == state_transition_.end()) {
      return false;
    }

    // 遍历current_state_的邻接状态
    for (auto it = state_transition_[current_state_].begin();
         it != state_transition_[current_state_].end(); ++it) {
      if (*it == state) {
        current_state_ = state;
        return true;
      }
    }
    return false;
  }
};
} // namespace libs