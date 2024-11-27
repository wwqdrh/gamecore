#pragma once

#include <map>
#include <string>
#include <utility>
#include <vector>
namespace gamedialog {
class DialogueWord {
private:
  std::string name;
  std::string text;
  std::string stage; // 这一句属于哪一个stage
  std::vector<std::pair<std::string, std::string>>
      responses; // key: response text, value: target
  std::vector<std::string> functions;

public:
  void set_stage(const std::string &n) { stage = n; }
  void set_name(const std::string &n) { name = n; }
  void set_text(const std::string &t) { text = t; }

  void add_response(const std::string &response_text,
                    const std::string &target) {
    responses.push_back({response_text, target});
  }

  void add_fn(const std::string &function_name) {
    functions.push_back(function_name);
  }

  // Getters
  const std::string &get_stage() const { return stage; }
  const std::string &get_name() const { return name; }
  const std::string &get_text() const { return text; }
  const std::vector<std::pair<std::string, std::string>> &
  get_responses() const {
    return responses;
  }
  const std::vector<std::string> &get_functions() const { return functions; }
};
} // namespace gamedialog