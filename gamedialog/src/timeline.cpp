// #include "godot_cpp/core/error_macros.hpp"
// #include "godot_cpp/variant/variant.hpp"
#include "word.h"
#include <memory>
// #include <variant>
#include <vector>

#include "flow.h"
#include "timeline.h"
// #include "wrappers.h"

namespace gamedialog {
// =====
// public
// =====
void Timeline::goto_begin() {
  if (current_ < stages.size()) {
    stages[current_]->clean();
    current_ = 0;
  }
}
void Timeline::goto_end() {
  if (current_ < stages.size()) {
    stages[current_]->clean();
    current_ = stages.size();
  }
}
void Timeline::skip_stage_count(int count) {
  if (current_ < stages.size()) {
    stages[current_]->clean();
    current_ += count;
    if (current_ > stages.size() - 1) {
      current_ = stages.size();
    }
  }
}
Timeline::Timeline(const std::string &data) {
  std::istringstream stream(data);
  std::string line;

  std::string tt = "";
  std::vector<std::string> cur_secions;
  while (std::getline(stream, line)) {
    if (line == "") {
      continue;
    }
    // 处理场景标记 [scene]
    if (line[0] == '[' && line.back() == ']') {
      if (!cur_secions.empty() && tt != "") {
        // 解析stage
        DiaStage node;
        for (auto cur : node.parse_all_stage(cur_secions)) {
          stages.push_back(cur);
          cur->set_timeline(this);
          stage_map[cur->get_stage_name()] = stages.size() - 1;
        }
        cur_secions.clear();
        tt = "";
      }
      tt = line.substr(1, line.length() - 2);
    }
    cur_secions.push_back(line);
  }

  // 处理最后一段对话
  if (!cur_secions.empty()) {
    // _parse_section(cur_stage, cur_names, cur_word);
    // 解析stage
    DiaStage node;
    for (auto cur : node.parse_all_stage(cur_secions)) {
      // WARN_PRINT(godot::vformat("add stage: %s",
      //                           godot::TO_GSTR(cur->get_stage_name())));
      stages.push_back(cur);
      cur->set_timeline(this);
      stage_map[cur->get_stage_name()] = stages.size() - 1;
    }
  }
}

std::string Timeline::current_stage() {
  if (current_ >= stages.size()) {
    return "";
  }
  auto cur = stages[current_];
  if (cur != nullptr) {
    return cur->get_stage_name();
  }
  return "";
}

bool Timeline::current_stage_is_doing() {
  if (current_ >= stages.size()) {
    return false;
  }
  auto cur = stages[current_];
  if (cur == nullptr) {
    return false;
  }
  return cur->is_doing();
}

bool Timeline::has_next() const {
  if (current_ >= stages.size()) {
    return false;
  }
  // 判断当前如果是标签，如果是:end那么就没有了
  return stages[current_]->has_next();
}

void Timeline::get_first_flag() {
  if (fn == nullptr) {
    return; // 使用默认的
  }

  // 如果是刚开头寻找新的stage
  // 找到第一个满足flag的stage
  for (auto item : stages) {
    if (check_stage_flag(item->get_stage_name())) {
      goto_stage(item->get_stage_name());
      return;
    }
  }
  // 没有一个满足条件的,直接end
  goto_end();
}

std::shared_ptr<DialogueWord> Timeline::next() {
  // 如果是刚开始那么需要找到满足flag的第一个
  if (current_ == 0 && stages.size() > 0 && stages[current_]->is_start()) {
    get_first_flag();
  } else if (!check_stage_flag(current_stage())) {
    // 如果不满足条件直接end
    goto_end();
  }

  if (!has_next()) {
    return nullptr;
  }
  std::string prev_stage = current_stage();
  auto result = stages[current_]->next();
  if (result) {
    return result;
  }
  std::string cur_stage = current_stage();
  if (prev_stage != cur_stage) {
    // 使用goto、skip等触发stage切换
    return next();
  }
  return nullptr;
}

std::vector<std::string> Timeline::all_stages() {
  std::vector<std::string> ret;
  for (auto item : stages) {
    ret.push_back(item->get_stage_name());
  }
  return ret;
}

bool Timeline::check_stage_flag(const std::string &stage) {
  if (fn == nullptr) {
    return true;
  }

  auto it = stage_map.find(stage);
  if (it == stage_map.end()) {
    return false;
  }
  auto stage_data = stages[it->second];
  auto flags = stage_data->get_flags();
  for (auto flag : flags) {
    if (!fn(flag)) {
      return false;
    }
  }
  return true;
}

void Timeline::goto_stage(const std::string &stage) {
  if (!check_stage_flag(stage)) {
    return;
  }
  if (stage_map.find(stage) == stage_map.end()) {
    return;
  }
  stages[current_]->clean();
  current_ = stage_map[stage];
}

int Timeline::stage_index(const std::string &label) {
  if (stage_map.find(label) == stage_map.end()) {
    return -1;
  }
  return stage_map[label];
}

std::vector<std::string> Timeline::get_available_stages() {
  std::vector<std::string> available_stages;
  for (auto item : stages) {
    if (check_stage_flag(item->get_stage_name())) {
      available_stages.push_back(item->get_stage_name());
    }
  }
  return available_stages;
}
} // namespace gamedialog