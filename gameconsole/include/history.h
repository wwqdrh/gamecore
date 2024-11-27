#pragma once
#include <memory>

#include "collection.h"
#include "lock.h"

namespace gameconsole {

class History : public Collection {
private:
  std::function<void(const std::string &)> _out;
  int max_length = 10;

  mutable ReentrantRWLock rwlock;

public:
  History(std::function<void(const std::string &)> out, int max_length = 10)
      : _out(out), max_length(max_length) {}

  void push(const Variant &value) {
    auto guard = rwlock.unique_lock();
    int l = size();
    if (l == max_length) {
      pop();
    }
    add(value);
    // last();
  }

  std::optional<Variant> pop() {
    auto guard = rwlock.unique_lock();

    std::optional<Variant> value = first();
    if (value.has_value()) {
      remove_by_index(0);
    }
    return value;
  }

  void print_all() {
    if (_out == nullptr) {
      return;
    }

    int i = 1;
    seek(0);
    {
      auto guard = rwlock.shared_lock();
      while (has_next()) {
        std::string command_name = std::get<std::string>(last().value());
        _out("[b]" + std::to_string(i) + ".[/b] [color=#ffff66][url=" +
             command_name + "]" + command_name + "[/url][/color]");
        i++;
      }
    }
  }
};
} // namespace gameconsole