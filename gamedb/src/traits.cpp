#include "traits.h"
#include <sstream>

namespace gamedb {
bool compareValues(const variant &v1, const variant &v2) {
  if (v1.index() != v2.index())
    return false;

  return std::visit(
      [](const auto &a, const auto &b) -> bool {
        using T1 = std::decay_t<decltype(a)>;
        using T2 = std::decay_t<decltype(b)>;
        if constexpr (std::is_same_v<T1, T2>) {
          return a == b;
        }
        return false;
      },
      v1, v2);
};

bool isInVector(const variant &single, const variant &vec) {
  return std::visit(
      [](const auto &value, const auto &container) -> bool {
        using ValueType = std::decay_t<decltype(value)>;
        using ContainerType = std::decay_t<decltype(container)>;

        if constexpr (std::is_same_v<ContainerType, std::vector<std::string>> &&
                      std::is_same_v<ValueType, std::string>) {
          return std::find(container.begin(), container.end(), value) !=
                 container.end();
        } else if constexpr (std::is_same_v<ContainerType, std::vector<int>> &&
                             std::is_same_v<ValueType, int>) {
          return std::find(container.begin(), container.end(), value) !=
                 container.end();
        } else if constexpr (std::is_same_v<ContainerType,
                                            std::vector<double>> &&
                             std::is_same_v<ValueType, double>) {
          return std::find(container.begin(), container.end(), value) !=
                 container.end();
        }
        return false;
      },
      single, vec);
}
variantDict variantDictFromJSON(const std::string &data) {
  rapidjson::Document doc;
  doc.Parse(data.c_str());
  if (doc.HasParseError()) {
    return {};
  }
  return variantDictFromValue(doc);
}
variantDict variantDictFromValue(const rapidjson::Value &data) {
  variantDict m = {};

  if (!data.IsObject()) {
    return m;
  }

  for (auto it = data.MemberBegin(); it != data.MemberEnd(); ++it) {
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
  return m;
}
double variantToDouble(const variant &data) {
  if (std::holds_alternative<int>(data)) {
    return static_cast<double>(std::get<int>(data));
  } else if (std::holds_alternative<double>(data)) {
    return std::get<double>(data);
  } else if (std::holds_alternative<std::string>(data)) {
    const std::string &str = std::get<std::string>(data);
    std::istringstream iss(str);
    double value;
    if (iss >> value) {
      return value;
    }
    return 0.0; // 转换失败，返回 0
  }
  return 0.0;
}
} // namespace gamedb