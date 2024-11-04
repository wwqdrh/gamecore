#pragma once

#include "gjson.h"
#include "rapidjson/rapidjson.h"
#include "traits.h"
#include <any>
#include <functional>
#include <iostream>
#include <map>
#include <memory>
#include <sstream>
#include <string>
#include <type_traits>
#include <typeindex>

#include <rapidjson/document.h>
#include <rapidjson/error/en.h>
#include <rapidjson/stringbuffer.h>
#include <rapidjson/writer.h>
#include <variant>
#include <vector>

namespace gamedb {
template <typename T> class DataClass {
private:
  std::map<std::string, std::function<void(T &, const variant &)>> setters;
  std::map<std::string, std::function<variant(const T &)>> getters;

  template <typename V> static V convert_any(const std::any &value) {
    if constexpr (std::is_same_v<V, std::vector<V>>) {
      return std::any_cast<std::vector<V>>(value);
    } else if constexpr (std::is_same_v<V, std::string>) {
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
        return std::any_cast<V>(value);
    } else {
      return std::any_cast<V>(value);
    }
  }

protected:
  template <typename M> void addMember(const std::string &name, M T::*member) {
    setters[name] = [member](T &obj, const variant &value) {
      obj.*member = std::get<M>(value); // convert_any<M>
    };
    getters[name] = [member](const T &obj) -> variant { return obj.*member; };
  }

public:
  void fromMap(const variantDict &data) {
    for (const auto &[key, value] : data) {
      if (setters.find(key) != setters.end()) {
        setters[key](*static_cast<T *>(this), value);
      }
    }
  }

  void fromJson(const std::string &data) { fromMap(variantDictFromJSON(data)); }

  void fromJsonValue(const rapidjson::Value &data) {
    fromMap(variantDictFromValue(data));
  }

  std::map<std::string, variant> toMap() const {
    std::map<std::string, variant> result;
    for (const auto &[key, getter] : getters) {
      result[key] = getter(*static_cast<const T *>(this));
    }
    return result;
  }

  std::string toJson() const {
    rapidjson::StringBuffer buffer;
    rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);

    writer.StartObject(); // 开始 JSON 对象

    std::map<std::string, variant> m = toMap();
    for (const auto &pair : m) {
      writer.Key(pair.first.c_str()); // 键
      // 根据类型写入对应的值
      if (std::holds_alternative<int>(pair.second)) {
        writer.Int(std::get<int>(pair.second));
      } else if (std::holds_alternative<double>(pair.second)) {
        writer.Double(std::get<double>(pair.second));
      } else if (std::holds_alternative<std::string>(pair.second)) {
        writer.String(std::get<std::string>(pair.second).c_str());
      } else if (std::holds_alternative<bool>(pair.second)) {
        writer.Bool(std::get<bool>(pair.second));
      } else if (std::holds_alternative<std::vector<int>>(pair.second)) {
        writer.StartArray();
        const auto &vec = std::get<std::vector<int>>(pair.second);
        for (const auto &elem : vec) {
          writer.Int(elem);
        }
        writer.EndArray();
      } else if (std::holds_alternative<std::vector<double>>(pair.second)) {
        writer.StartArray();
        const auto &vec = std::get<std::vector<double>>(pair.second);
        for (const auto &elem : vec) {
          writer.Double(elem);
        }
        writer.EndArray();
      } else if (std::holds_alternative<std::vector<std::string>>(
                     pair.second)) {
        writer.StartArray();
        const auto &vec = std::get<std::vector<std::string>>(pair.second);
        for (const auto &elem : vec) {
          writer.String(elem.c_str());
        }
        writer.EndArray();
      }
    }

    writer.EndObject(); // 结束 JSON 对象

    return buffer.GetString(); // 将缓冲区中的数据转换为字符串
  }

  rapidjson::Value toJsonValue() const {
    rapidjson::Document doc;
    std::map<std::string, std::any> m = toMap();
    return GJson::toValue(m, doc.GetAllocator());
  }

  static std::vector<std::shared_ptr<T>> fromJsonArr(const std::string &data) {
    std::vector<std::shared_ptr<T>> result;

    rapidjson::Document document;
    document.Parse(data.c_str());
    if (document.HasParseError()) {
      std::cerr << "JSON parse error: "
                << GetParseError_En(document.GetParseError()) << std::endl;
      return result;
    }
    if (!document.IsArray()) {
      std::cerr << "Expected a JSON array" << std::endl;
      return result;
    }
    for (auto it = document.Begin(); it != document.End(); ++it) {
      if (it == nullptr) {
        continue;
      }
      T item;
      item.fromJsonValue(*it);
      result.push_back(std::make_shared<T>(item));
    }
    return result;
  }

  static std::vector<std::shared_ptr<T>>
  fromJsonValueArr(const rapidjson::Value &data) {
    std::vector<std::shared_ptr<T>> result;

    if (!data.IsArray()) {
      std::cerr << "Expected a JSON array" << std::endl;
      return result;
    }
    for (auto it = data.Begin(); it != data.End(); ++it) {
      if (it == nullptr) {
        continue;
      }
      T item;
      item.fromJsonValue(*it);
      result.push_back(std::make_shared<T>(item));
    }
    return result;
  }

  static std::string toJsonArr(const std::vector<std::shared_ptr<T>> &data) {
    std::string result = "[";
    for (size_t i = 0; i < data.size(); i++) {
      result += data[i]->toJson();
      if (i != data.size() - 1) {
        result += ",";
      }
    }
    result += "]";
    return result;
  }
};
} // namespace gamedb