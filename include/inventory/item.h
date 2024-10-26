#pragma once

#include <map>
#include <string>
#include <variant>
#include <vector>
namespace gamedb {
class GoodItem {
public:
  using variant = std::variant<int, std::string, bool>;

public:
  std::string name = "";
  int count = 0;

private:
  std::map<std::string, variant> ext_info;

public:
  friend class Slot;
  GoodItem() = default;
  GoodItem(const std::string &name, int count) : name(name), count(count) {}
  GoodItem(const std::string &name, int count, std::vector<std::string> exts)
      : name(name), count(count) {
    for (auto item : exts) {
      add_ext(item);
    }
  }
  void add_ext(const std::string &name) { ext_info[name] = ""; }
  void set_ext(const std::string &name, variant val) {
    auto it = ext_info.find(name);
    if (it != ext_info.end()) {
      ext_info[name] = val;
    }
  }
  bool check_ext(const std::string &name, variant val) {
    auto it = ext_info.find(name);
    if (it == ext_info.end()) {
      return false;
    }
    return it->second == val;
  }
};
} // namespace gamedb