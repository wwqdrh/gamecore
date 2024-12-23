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
    auto word = std::get<std::shared_ptr<DialogueWord>>(dialogue_keys[current_++]);
    // Execute all functions for this dialogue word
    for (const auto& fn : word->get_functions()) {
      if (starts_with(fn, "set:")) {
        std::string var_expr = trim_prefix(fn, "set:");
        auto parts = split(var_expr, '=');
        if (parts.size() == 2) {
          set_variable(strip(parts[0]), strip(parts[1]));
        }
      }
      else if (starts_with(fn, "if:")) {
        // Handle conditional jump
        std::string expr = trim_prefix(fn, "if:");
        auto parts = split(expr, ':');
        if (parts.size() == 3) {
          auto conditions = strip(parts[0]);
          auto true_label = strip(parts[1]);
          auto false_label = strip(parts[2]);
          
          if (check_conditions(conditions)) {
            goto_label(true_label);
          } else {
            goto_label(false_label);
          }
        }
      } else if (starts_with(fn, "goto:")) {
        std::string label = trim_prefix(fn, "goto:");
        goto_label(label);
      }
    }
    return word;
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
  std::string var_block;
  bool in_var_block = false;

  for (auto line : data) {
    // Skip comments and empty lines
    if (line[0] == '#' || is_empty(strip(line))) {
      continue;
    }

    // Handle variable block
    if (line == "```") {
      if (!in_var_block) {
        in_var_block = true;
        continue;
      } else {
        in_var_block = false;
        _parse_variables(var_block);
        continue;
      }
    }

    if (in_var_block) {
      var_block += line + "\n";
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
  if (!cur_word.empty()) {
    if (cur_names.size() == 0) {
      _parse_section({""}, cur_word);
    } else {
      _parse_section(cur_names, cur_word);
    }
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
        std::string fn_expr = trim_prefix(word, "@");
        // Handle label setting immediately
        if (starts_with(fn_expr, "label:")) {
          std::string label = trim_prefix(fn_expr, "label:");
          set_label(strip(label), dialogue_keys.size());
        }
        cur->add_fn(fn_expr);
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

std::string DiaStage::get_variable(const std::string &key) const {
  auto it = scene_variables_.find(key);
  return it != scene_variables_.end() ? it->second : "";
}

void DiaStage::_parse_variables(const std::string &var_block) {
  std::stringstream ss(var_block);
  std::string line;

  while (std::getline(ss, line)) {
    line = strip(line);
    if (line.empty() || line[0] == '#')
      continue;

    auto pos = line.find('=');
    if (pos != std::string::npos) {
      std::string key = strip(line.substr(0, pos));
      std::string value = strip(line.substr(pos + 1));
      if (!key.empty()) {
        scene_variables_[key] = value;
      }
    }
  }
}
} // namespace gamedialog