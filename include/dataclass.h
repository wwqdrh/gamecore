#pragma once

#include <any>
#include <functional>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <typeindex>

#include <rapidjson/document.h>
#include <rapidjson/error/en.h>
#include <rapidjson/stringbuffer.h>
#include <rapidjson/writer.h>

namespace gamedb {
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
        return std::any_cast<V>(value);
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

  void fromJson(const std::string &data) {
    rapidjson::Document document;
    document.Parse(data.c_str());

    if (document.HasParseError()) {
      std::cerr << "JSON parse error: "
                << GetParseError_En(document.GetParseError()) << std::endl;
      return;
    }

    if (!document.IsObject()) {
      std::cerr << "Expected a JSON object" << std::endl;
      return;
    }

    std::map<std::string, std::any> m;
    for (auto it = document.MemberBegin(); it != document.MemberEnd(); ++it) {
      std::string key = it->name.GetString();

      // 根据不同类型解析值
      if (it->value.IsInt()) {
        m[key] = it->value.GetInt();
      } else if (it->value.IsDouble()) {
        m[key] = it->value.GetDouble();
      } else if (it->value.IsString()) {
        m[key] = it->value.GetString();
      } else if (it->value.IsBool()) {
        m[key] = it->value.GetBool();
      }
    }

    fromMap(m);
  }

  std::map<std::string, std::any> toMap() const {
    std::map<std::string, std::any> result;
    for (const auto &[key, getter] : getters) {
      result[key] = getter(*static_cast<const T *>(this));
    }
    return result;
  }

  std::string toJson() const {
    rapidjson::StringBuffer buffer;
    rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);

    writer.StartObject(); // 开始 JSON 对象

    std::map<std::string, std::any> m = toMap();
    for (const auto &pair : m) {
      writer.Key(pair.first.c_str()); // 键
      // 根据类型写入对应的值
      if (pair.second.type() == typeid(int)) {
        writer.Int(std::any_cast<int>(pair.second));
      } else if (pair.second.type() == typeid(double)) {
        writer.Double(std::any_cast<double>(pair.second));
      } else if (pair.second.type() == typeid(std::string)) {
        writer.String(std::any_cast<std::string>(pair.second).c_str());
      } else if (pair.second.type() == typeid(bool)) {
        writer.Bool(std::any_cast<bool>(pair.second));
      }
    }

    writer.EndObject(); // 结束 JSON 对象

    return buffer.GetString(); // 将缓冲区中的数据转换为字符串
  }

  static std::vector<T> fromJsonArr(const std::string &data) {
    std::vector<T> result;

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
      rapidjson::StringBuffer buffer;
      rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
      it->Accept(writer);

      T item;
      item.fromJson(buffer.GetString());
      result.push_back(item);
    }
    return result;
  }

  static std::string toJsonArr(const std::vector<T> &data) {
    std::string result = "[";
    for (size_t i = 0; i < data.size(); i++) {
      result += data[i].toJson();
      if (i != data.size() - 1) {
        result += ",";
      }
    }
    result += "]";
    return result;
  }
};
} // namespace gamedb