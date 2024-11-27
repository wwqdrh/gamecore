#pragma once
#include <memory>
#include <sstream>

#include "collection.h"
#include "types.h"

namespace gameconsole {
class Argument {
private:
  Variant _normalized_value;
  std::string _original_value;

public:
  std::string name;
  std::shared_ptr<BaseType> type_;
  std::string description;

public:
  Argument() = default;

  Argument(const std::string &p_name, ValueType p_type,
           const std::string &p_description)
      : name(p_name), type_(TypeFactory::build(p_type)),
        description(p_description) {}

  std::string get_value() { return _original_value; }
  Variant get_normalized_value() { return _normalized_value; }
  CheckResult set_value(const std::string &value) {
    _original_value = value;
    auto check = type_->check(value);
    if (check == CheckResult::Ok) {
      _normalized_value = type_->normalize(_original_value);
    }
    return check;
  }
  std::string describe() {
    std::stringstream ss;
    std::string expire = type_.use_count() >= 0 ? type_->toString() : "null";
    ss << "<" << name << ":" << expire << ">";
    return ss.str();
  }
};
}