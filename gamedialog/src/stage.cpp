#include "stage.h"
#include <vector>

namespace gamedialog {

DiaStage::DiaStage(const std::string &data) {
  std::stringstream ss(data);
  std::string line;

  std::vector<std::string> data_list;
  while (std::getline(ss, line)) {
    data_list.push_back(line);
  }

  initial(data_list);
}

bool DiaStage::has_next() const {
  if (current_ >= dialogue_keys.size()) {
    return false;
  }
  if (std::holds_alternative<std::shared_ptr<ControlFlow>>(
          dialogue_keys[current_])) {
    if (auto ptr =
            std::get<std::shared_ptr<ControlFlow>>(dialogue_keys[current_])) {
      if (timeline_ != nullptr) {
        return ptr->hasNext(*timeline_);
      }
    }
  }
  return true;
}

std::shared_ptr<DialogueWord> DiaStage::next() {
  auto cur = dialogue_keys[current_];
  if (std::holds_alternative<std::shared_ptr<DialogueWord>>(cur)) {
    return std::get<std::shared_ptr<DialogueWord>>(dialogue_keys[current_++]);
  } else if (std::holds_alternative<std::shared_ptr<ControlFlow>>(cur)) {
    // 说明当前是controlflow
    auto flow = std::get<std::shared_ptr<ControlFlow>>(cur);
    if (timeline_) {
      flow->execute(*timeline_);
    }
    return nullptr;
  } else {
    return nullptr;
  }
}

void DiaStage::initial(const std::vector<std::string> &data) {
  std::vector<std::string> cur_names;
  std::vector<std::string> cur_word;
  for (auto line : data) {
    // 跳过注释和空行
    if (line[0] == '#' || is_empty(strip(line))) {
      continue;
    }

    // 处理场景标记 [scene], 可能存在标记，例如[stage@flag1;flag2]
    if (line[0] == '[' && line.back() == ']') {
      auto stage_parts = split(line.substr(1, line.length() - 2), '@');
      stage_name = stage_parts[0];
      if (stage_parts.size() == 2) {
        stage_flags = split(stage_parts[1], ';');
      }
    }
    // 处理角色名称 (name1,name2)
    else if (line[0] == '(' && line.back() == ')') {
      if (!cur_names.empty() && !cur_word.empty()) {
        _parse_section(cur_names, cur_word);
        cur_names.clear();
        cur_word.clear();
      }
      std::string names_str = line.substr(1, line.length() - 2);
      cur_names = split(names_str, ',');
    }
    // 处理对话内容
    else {
      cur_word.push_back(line);
    }
  }
  // 处理最后一段对话
  if (!cur_names.empty() && !cur_word.empty()) {
    _parse_section(cur_names, cur_word);
  }
}

void DiaStage::_parse_section(const std::vector<std::string> &names,
                              const std::vector<std::string> &words) {
  int curindex = 0;
  std::shared_ptr<DialogueWord> cur = nullptr;

  for (const auto &word : words) {
    // 跳过注释和空行
    if (starts_with(word, "#") || word.empty()) {
      continue;
    }

    // 处理响应选项
    if (starts_with(word, "-")) {
      if (cur != nullptr) {
        std::string response = trim_prefix(word, "-");
        auto parts = split(response, '@');
        if (parts.size() == 2) {
          cur->add_response(parts[0], parts[1]);
        }
      }
    }
    // 处理函数调用
    else if (starts_with(word, "@")) {
      if (cur != nullptr) {
        cur->add_fn(trim_prefix(word, "@"));
      }
    }
    // 控制流程
    else if (starts_with(word, ":")) {
      auto flow = ControlFlowFactory::createFromString(word);
      flow->set_stage_name(stage_name);
      dialogue_keys.push_back(flow);
    }
    // 处理普通对话
    else {
      cur = std::make_shared<DialogueWord>();
      cur->set_stage(stage_name);
      cur->set_name(names[curindex]);
      cur->set_text(trim_suffix(word, "+"));
      dialogue_keys.push_back(cur);

      if (!ends_with(word, "+")) {
        curindex = (curindex + 1) % names.size();
      }
    }
  }
}
} // namespace gamedialog