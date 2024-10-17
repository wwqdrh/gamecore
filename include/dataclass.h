#pragma once

#include <any>
#include <functional>
#include <iostream>
#include <map>
#include <string>
#include <typeindex>

template <typename T> class DataClass {
private:
  std::map<std::string, std::function<void(T &, const std::any &)>> setters;
  std::map<std::string, std::function<std::any(const T &)>> getters;

  template <typename V> static V convert_any(const std::any &value) {
    if constexpr (std::is_same_v<V, std::string>) {
      if (value.type() == typeid(const char *))
        return std::string(std::any_cast<const char *>(value));
      else
        return std::any_cast<std::string>(value);
    } else if constexpr (std::is_arithmetic_v<V>) {
      if (value.type() == typeid(int))
        return static_cast<V>(std::any_cast<int>(value));
      else if (value.type() == typeid(long))
        return static_cast<V>(std::any_cast<long>(value));
      else if (value.type() == typeid(float))
        return static_cast<V>(std::any_cast<float>(value));
      else if (value.type() == typeid(double))
        return static_cast<V>(std::any_cast<double>(value));
      else
        throw std::bad_any_cast();
    } else {
      return std::any_cast<V>(value);
    }
  }

protected:
  template <typename M> void addMember(const std::string &name, M T::*member) {
    setters[name] = [member](T &obj, const std::any &value) {
      obj.*member = convert_any<M>(value);
    };
    getters[name] = [member](const T &obj) -> std::any { return obj.*member; };
  }

public:
  void fromMap(const std::map<std::string, std::any> &data) {
    for (const auto &[key, value] : data) {
      if (setters.find(key) != setters.end()) {
        setters[key](*static_cast<T *>(this), value);
      }
    }
  }

  std::map<std::string, std::any> toMap() const {
    std::map<std::string, std::any> result;
    for (const auto &[key, getter] : getters) {
      result[key] = getter(*static_cast<const T *>(this));
    }
    return result;
  }
};