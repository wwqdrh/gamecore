#pragma once

#include "argument.h"
#include "collection.h"
#include "types.h"
#include <functional>
#include <memory>
#include <variant>
#include <vector>

#include "lock.h"

namespace gameconsole {

class Command {
private:
  mutable ReentrantRWLock rwlock;

public:
  std::string name;
  std::function<Variant(std::vector<Variant>)> target;
  std::vector<std::shared_ptr<Argument>> arguments;
  std::string description;

  Command() = default;
  Command(const std::string &p_name,
          const std::function<Variant(std::vector<Variant>)> &p_target,
          const std::vector<std::shared_ptr<Argument>> &p_arguments,
          const std::string &p_description)
      : name(p_name), target(p_target), arguments(p_arguments),
        description(p_description) {}

  Variant execute(std::vector<std::string> inArgs) const {
    auto guard = rwlock.shared_lock();

    CheckResult argAssig;
    std::vector<Variant> args;
    int i = 0;
    while (i < arguments.size() && i < inArgs.size()) {
      argAssig = arguments[i]->set_value(inArgs[i]);
      if (argAssig == CheckResult::Failed) {
        return false;
      } else if (argAssig == CheckResult::Canceled) {
        return true;
      }
      args.push_back(arguments[i]->get_normalized_value());
      i++;
    }
    return target(args);
  }
};

class Commands : public Collection {
public:
  Commands() = default;
  Commands(const VariantMap &value)
      : Collection(std::make_shared<VariantMap>(value)) {}

  std::shared_ptr<Collection> find(const std::string &name) const {
    return filter([name](const Variant &key, const Variant &value, int index,
                         const Collection &coll) -> bool {
      // 判断是否以name开头
      if (std::holds_alternative<std::string>(key)) {
        return std::get<std::string>(key).find(name) == 0;
      }
      return false;
    });
  }
};
} // namespace gameconsole