#pragma once

#include "rapidjson/document.h"
#include <map>
#include <type_traits>

namespace gamedb {
template <typename T> struct always_false : std::false_type {};

// 判断variant类型
// 添加 variant 类型检查的辅助模板
template <typename T> struct is_variant : std::false_type {};

template <typename... Args>
struct is_variant<std::variant<Args...>> : std::true_type {};

// ====
// 是否为vector类型
// ====
template <typename T> struct is_vector : std::false_type {};
template <typename T, typename A>
struct is_vector<std::vector<T, A>> : std::true_type {};
template <typename T> struct is_vector<std::vector<T>> : std::true_type {};

template <typename V> struct is_string_vector : std::false_type {};

template <>
struct is_string_vector<std::vector<std::string>> : std::true_type {};

template <typename V> struct is_int_vector : std::false_type {};

template <> struct is_int_vector<std::vector<int>> : std::true_type {};

// ====
// 是否为map类型
// ====
template <typename T> struct is_map : std::false_type {};
template <typename K, typename V, typename C, typename A>
struct is_map<std::map<K, V, C, A>> : std::true_type {};
template <typename K, typename V>
struct is_map<std::map<K, V>> : std::true_type {};

// 是否为unordered_map
template <typename T> struct is_unordered_map : std::false_type {};
template <typename K, typename V, typename H, typename E, typename A>
struct is_unordered_map<std::unordered_map<K, V, H, E, A>> : std::true_type {};

// 检查类是否有 toJson 方法
template <typename T, typename = void> struct has_to_json : std::false_type {};
template <typename T>
struct has_to_json<T,
                   std::void_t<decltype(std::declval<T>().toJson(
                       std::declval<rapidjson::Document::AllocatorType &>()))>>
    : std::true_type {};

// 检查是否有fromJson 的static方法
// 定义一个 trait 来检查类是否可以从 JSON 反序列化
template <typename T, typename = void>
struct has_from_json : std::false_type {};
template <typename T>
struct has_from_json<T, std::void_t<decltype(T::fromJson(
                            std::declval<const rapidjson::Value &>()))>>
    : std::true_type {};

// 将任意类型转换为字符串的辅助函数
template <typename T> std::string toString(const T &value) {
  if constexpr (std::is_same_v<T, std::string>) {
    return value;
  } else if constexpr (std::is_same_v<T, const char *>) {
    return std::string(value);
  } else if constexpr (std::is_arithmetic_v<T>) {
    return std::to_string(value);
  } else {
    static_assert(always_false<T>::value,
                  "Unsupported key type for map conversion");
  }
}
} // namespace gamedb