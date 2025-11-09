#include <fstream>
#include <iostream>
#include <memory>
#include <mutex>
#include <shared_mutex>
#include <sstream>
#include <unordered_set>
#include <utility>
#include <vector>

#include "godot_cpp/core/error_macros.hpp"
#include "rapidjson/document.h"
#include "rapidjson/rapidjson.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"

#include "gjson.h"
#include "glogger.h"
#include "traits.h"

using namespace rapidjson;

namespace gamedb {

// =============
// public
// =============
Value *GJson::query_value(const std::string &field) const {
  auto lock = rwlock.shared_lock();
  GLOG_DEBUG("GJson", "Entering query_value with field: " + field);

  if (raw_data.IsNull()) {
    GLOG_ERROR("GJson", "raw_data is null");
    return nullptr;
  }

  std::vector<std::string> parts = split(field, ';');
  Value *current = const_cast<Value *>(static_cast<const Value *>(&raw_data));
  GLOG_DEBUG("GJson", "Initial current pointer status: " +
                          std::string(current ? "valid" : "null"));

  for (const auto &part : parts) {
    if (current == nullptr || part.empty()) {
      GLOG_WARNING("GJson",
                   "Current pointer is null or part is empty. Part: " + part);
      return nullptr;
    }

    if (part[0] == '#') {
      GLOG_DEBUG("GJson", "Found special operator '#' in part: " + part);
      return nullptr;
    } else {
      GLOG_DEBUG("GJson", "Traversing with part: " + part);
      Value *next = traverse(*current, part);
      if (next == nullptr) {
        GLOG_WARNING("GJson", "Traverse returned null for part: " + part);
        return nullptr;
      }
      current = next;
    }
  }

  GLOG_DEBUG("GJson", "Successfully found value for field: " + field);
  return current;
}

// #condition, 会检查#include、#exclude
// #branch, condition:tag,...
Value GJson::query_value_dynamic(const std::string &field) const {
  auto lock = rwlock.shared_lock();

  std::vector<std::string> parts = split(field, ';');
  // Value curr_data;
  // curr_data.CopyFrom(raw_data, raw_data.GetAllocator());
  // Value *current = &raw_data;
  Value *current = const_cast<Value *>(static_cast<const Value *>(&raw_data));

  Value temp;
  temp.SetArray();
  Value str_temp;
  for (const auto &part : parts) {
    if (current == nullptr) {
      break;
    }
    if (part.empty())
      continue;

    if (part[0] == '#') {
      // Special operation
      // example1: #(a=1)
      // example3: #last('age' | '', '>', 10),
      // 对于数组来说，如果第一个参数有值那么就是针对数组[字典]，寻找字典中满足大于10的最后一个元素，否组就是当前数组中的大于10的最后一个元素
      // example3: #first('age' | '', '>', 10), 同上
      // #condition({"val": 1})
      // example3: #random
      // Special operation
      // 将part字符串()中的提取出来，并且使用,分割，并且获取#last()括号之前的字符串
      int idx = part.find('(');
      std::string operation = part.substr(1, idx - 1);
      std::vector<std::string> ops =
          split(part.substr(part.find('(') + 1, part.length() - idx - 2), '|');
      if (operation == "all") {
        temp.Clear();
        temp.SetArray();
        for (size_t i = 0; i < current->Size(); ++i) {
          Value *t = getCompareElements(current->operator[](i), ops[0], ops[1],
                                        ops[2], false);
          if (t != nullptr) {
            Value tt;
            tt.CopyFrom(*t, raw_data.GetAllocator());
            temp.PushBack(tt, raw_data.GetAllocator());
          }
        }
        current = &temp;
        continue;
      } else if (operation == "last") {
        current = getCompareElements(*current, ops[0], ops[1], ops[2], true);
        continue;
      } else if (operation == "first") {
        current = getCompareElements(*current, ops[0], ops[1], ops[2], false);
        continue;
      } else if (operation == "random") {
        // temp.Clear();
        // temp.SetArray();
        size_t count = std::stoul(ops[0]);
        temp = getRandomElements(*current, count);
        current = &temp;
        break;
      } else if (operation == "keys") {
        if (current->IsArray()) {
          temp.Clear();
          temp.SetArray();

          for (int i = 0; i < ops.size(); i++) {
            std::string key = ops[i];
            Value *t = traverse(*current, key);
            if (t != nullptr) {
              Value tt;
              tt.CopyFrom(*t, raw_data.GetAllocator());
              temp.PushBack(tt, raw_data.GetAllocator());
            }
          }

          current = &temp;
          break;
        } else if (current->IsObject()) {
          temp.Clear();
          temp.SetArray();
          if (ops.size() == 0) {
            temp = get_keys(*current);
          } else {
            for (int i = 0; i < ops.size(); i++) {
              std::string key = ops[i];
              Value *t = traverse(*current, key);
              if (t != nullptr) {
                Value objectitem;
                objectitem.SetObject();
                Value name;
                name.SetString(key.c_str(), raw_data.GetAllocator());
                Value value;
                value.CopyFrom(*t, raw_data.GetAllocator());
                objectitem.AddMember(name, value, raw_data.GetAllocator());
                temp.PushBack(objectitem, raw_data.GetAllocator());
              }
            }
          }
          current = &temp;
          break;
        }

      } else if (operation == "condition") {
        if (current->IsArray()) {
          temp.Clear();
          temp.SetArray();
          for (size_t i = 0; i < current->Size(); ++i) {
            Value *t = checkCondition_(current->operator[](i), ops[0]);
            if (t != nullptr) {
              Value tt;
              tt.CopyFrom(*t, raw_data.GetAllocator());
              temp.PushBack(tt, raw_data.GetAllocator());
            }
          }
          current = &temp;
          break;
        } else if (current->IsObject()) {
          Value *t = checkCondition_(*current, ops[0]);
          if (t == nullptr) {
            current = nullptr;
            break;
          }
        } else {
          current = nullptr;
          break;
        }

      } else if (operation == "branch") {
        // 判断current是不是object，是的话看有没有#branch分支，有的话遍历并进行检查
        if (current->IsObject() && current->HasMember("#branch") &&
            current->operator[]("#branch").IsArray()) {
          temp.Clear();
          temp.SetArray();
          auto v = current->operator[]("#branch").GetArray();
          variantDict cur_state = variantDictFromJSON(ops[0]);
          for (size_t i = 0; i < v.Size(); ++i) {
            std::string name = v[i].GetString();
            std::vector<std::string> name_parts = split(name, ':');
            if (name_parts.size() == 2) {
              // 第一部分是条
              if (!condition_.checkCondition(cur_state, name_parts[0])) {
                // 不满足条件，不进入这个分支
                continue;
              }
              name = name_parts[1];
            } else if (name_parts.size() == 1) {
              name = name_parts[0];
            }
            Value vitem = query_value_dynamic(name);
            // 满足条件了也需要检查
            Value *t = checkCondition_(vitem, ops[0]);
            if (t != nullptr) {
              Value tt;
              tt.CopyFrom(*t, raw_data.GetAllocator());
              temp.PushBack(tt, raw_data.GetAllocator());
            }
          }
          current = &temp;
          continue;
        } else {
          current = nullptr;
          break;
        }
      } else if (operation == "weight") {
        // ["100*1", 1001, "1002*9", 1003*9999]
        // 计算概率，并且使用10二进制对应某个位置上是否参与计算

        if (!current->IsObject() || !current->HasMember("#weight") ||
            !current->operator[]("#weight").IsArray()) {
          current = nullptr;
          break;
        }

        std::vector<bool> joined;
        if (ops.size() > 0) {
          for (auto ch : ops[0]) {
            joined.push_back(ch == '1');
          }
        }
        auto v = current->operator[]("#weight").GetArray();
        double _event_weight_total = 0;
        std::vector<std::pair<std::string, double>> events;
        for (size_t i = 0; i < v.Size(); ++i) {
          if (i < joined.size() && !joined[i]) {
            continue;
          }
          double weight = 1.0;
          std::string eid = "";
          if (v[i].IsString()) {
            std::vector<std::string> parts = split(v[i].GetString(), '*');
            eid = parts[0];
            if (parts.size() == 2) {
              weight = variantToDouble(parts[1]);
            }
            _event_weight_total += weight;

          } else if (v[i].IsInt()) {
            eid = std::to_string(v[i].GetInt());
          } else if (v[i].IsDouble()) {
            eid = std::to_string(int(v[i].GetDouble()));
          }
          events.push_back({eid, weight});
        }
        if (events.size() == 0) {
          current = nullptr;
          break;
        }

        std::random_device rd;  // 使用硬件生成随机数种子
        std::mt19937 gen(rd()); // 使用 Mersenne Twister 19937 引擎
        // 定义范围 1 到 n 的均匀分布
        std::uniform_int_distribution<> distrib(1, int(_event_weight_total));
        // 生成随机数
        int r = distrib(gen);
        bool found = false;
        for (auto &event : events) {
          if (r < event.second) {
            str_temp.SetString(event.first.c_str(), raw_data.GetAllocator());
            current = &str_temp;
            found = true;
            break;
          }
        }

        if (!found) {
          str_temp.SetString(events.back().first.c_str(),
                             raw_data.GetAllocator());
          current = &str_temp;
          break;
        }
      }
    } else {
      // Normal key or index
      Value *next = traverse(*current, part);
      current = next;
      if (next == nullptr) {
        break;
      }
    }
  }
  // std::cout << current->Size() << std::endl;
  if (current != nullptr) {
    StringBuffer buffer;
    Writer<StringBuffer> writer(buffer);
    current->Accept(writer);
    return toValue(buffer.GetString());
  } else {
    return toValue("");
  }
}

// 解析传入的查询参数
// 首先使用;符号分割成查询链条
// 1、每一个部分，如果不以#开头，则就是简单的key参数，如果是数字就是对应的位置的元素
// 2、每一个部分，如果以#开头，那么就是特殊操作，例如#random:10，就是指在当前的object(需要适配{}和[])中随机获取10个元素
// 3、如果是#开头然后包含一个括号(),
// 那么括号里面的就是一个解析参数，例如#(@=1)就是判断前的object的key==1,
// 注意这里都是字符串
std::string GJson::query(const std::string &field) const {
  auto lock = rwlock.shared_lock();

  Value current = query_value_dynamic(field);
  if (current.IsNull()) {
    return "";
  }
  // std::cout << current->Size() << std::endl;
  StringBuffer buffer;
  Writer<StringBuffer> writer(buffer);
  current.Accept(writer);
  return buffer.GetString();
}

std::vector<std::string> GJson::keys(const std::string &field) const {
  auto write = rwlock.shared_lock();

  Value *current = query_value(field);
  if (current == nullptr) {
    return {};
  }
  std::vector<std::string> result;
  if (current->IsArray()) {
    for (size_t i = 0; i < current->Size(); ++i) {
      result.push_back(std::to_string(i));
    }
  } else if (current->IsObject()) {
    for (auto it = current->MemberBegin(); it != current->MemberEnd(); ++it) {
      result.push_back(it->name.GetString());
    }
  }
  return result;
}

std::vector<std::string> GJson::values(const std::string &field) const {
  auto write = rwlock.shared_lock();

  Value *current = query_value(field);
  if (current == nullptr) {
    return {};
  }
  std::vector<std::string> result;
  Value *val;
  if (current->IsArray()) {
    for (size_t i = 0; i < current->Size(); ++i) {
      val = &current->operator[](i);
      StringBuffer buffer;
      Writer<StringBuffer> writer(buffer);
      val->Accept(writer);
      result.push_back(buffer.GetString());
    }
  } else if (current->IsObject()) {
    for (auto it = current->MemberBegin(); it != current->MemberEnd(); ++it) {
      val = &it->value;
      StringBuffer buffer;
      Writer<StringBuffer> writer(buffer);
      val->Accept(writer);
      result.push_back(buffer.GetString());
    }
  }
  return result;
}

// =======
// private
// =======
std::vector<std::string> GJson::split(const std::string &s,
                                      char delimiter) const {
  std::vector<std::string> tokens;
  std::string token;
  std::istringstream tokenStream(s);
  while (std::getline(tokenStream, token, delimiter)) {
    tokens.push_back(token);
  }
  return tokens;
}

Value *GJson::traverse(Value &current, const std::string &key) const {
  GLOG_DEBUG("GJson", "Entering traverse with key: " + key);

  if (current.IsObject()) {
    GLOG_DEBUG("GJson", "Current is object, checking for key: " + key);
    if (current.HasMember(key.c_str())) {
      return &current[key.c_str()];
    }
  } else if (current.IsArray()) {
    GLOG_DEBUG("GJson", "Current is array, checking index: " + key);
    if (!key.empty() && std::all_of(key.begin(), key.end(), ::isdigit)) {
      size_t index = std::stoul(key);
      if (index < current.Size()) {
        return &current[index];
      }
      GLOG_WARNING("GJson", "Array index out of bounds: " + key +
                                ", size: " + std::to_string(current.Size()));
    }
  }

  GLOG_WARNING("GJson", "Traverse failed for key: " + key);
  return nullptr;
}

Value GJson::get_keys(Value &current) const {
  Value res;
  res.SetArray();
  if (current.IsObject()) {
    std::vector<Value::MemberIterator> members;
    for (auto it = current.MemberBegin(); it != current.MemberEnd(); ++it) {
      Value name(it->name, raw_data.GetAllocator());
      res.PushBack(name, raw_data.GetAllocator());
    }
  }
  return res;
}

Value GJson::getRandomElements(Value &current, size_t count) const {
  Document doc;
  if (current.IsArray()) {
    std::vector<size_t> indices(current.Size());
    std::iota(indices.begin(), indices.end(), 0);
    std::random_device rd;
    std::mt19937 g(rd());
    std::shuffle(indices.begin(), indices.end(), g);
    size_t n = std::min(count, indices.size());

    if (n == 1) {
      Value curr;
      curr.CopyFrom(current[indices[0]], doc.GetAllocator());
      return curr;
    } else {
      Value result(kArrayType);
      for (size_t i = 0; i < n; ++i) {
        Value curr;
        curr.CopyFrom(current[indices[i]], doc.GetAllocator());
        result.PushBack(curr, doc.GetAllocator());
      }
      return result;
    }
  } else if (current.IsObject()) {
    std::vector<Value::MemberIterator> members;
    for (auto it = current.MemberBegin(); it != current.MemberEnd(); ++it) {
      members.push_back(it);
    }

    std::random_device rd;
    std::mt19937 g(rd());
    std::shuffle(members.begin(), members.end(), g);

    Value o(kObjectType);
    size_t n = std::min(count, members.size());
    std::cout << n << std::endl;
    for (size_t i = 0; i < n; ++i) {
      Value name(members[i]->name, doc.GetAllocator());
      Value value;
      value.CopyFrom(members[i]->value, doc.GetAllocator());
      o.AddMember(name, value, doc.GetAllocator());
    }
    return o;
  }
  std::cout << "unknown operator" << std::endl;
  Value s;
  s.SetString("nodata");
  return s;
}
Value *GJson::checkCondition_(Value &current, const std::string &data) const {
  if (!current.IsObject()) {
    return nullptr;
  }

  if ((!current.HasMember("#include") || !current["#include"].IsString()) &&
      (!current.HasMember("#exclude") || !current["#exclude"].IsString())) {
    // 两个标签都没有就直接返回
    return &current;
  }

  // #include判断是否包含，#exclude判断是否不包含，需要两个条件同时为true才返回
  variantDict data_dict = variantDictFromJSON(data);
  if (current.HasMember("#include") && current["#include"].IsString()) {
    std::string cond = current["#include"].GetString();
    if (!condition_.checkCondition(data_dict, cond)) {
      return nullptr;
    }
  }
  if (current.HasMember("#exclude") && current["#exclude"].IsString()) {
    std::string exclude_cond = current["#exclude"].GetString();
    if (condition_.checkCondition(data_dict, exclude_cond)) {
      return nullptr; // exclude满足条件则排除在外
    }
  }
  return &current;
}
Value *GJson::getCompareElements(Value &current, const std::string &key,
                                 const std::string &op,
                                 const std::string &value, bool rindex) const {
  if (current.IsArray()) {
    // 如果rindex为true，那么需要反向遍历
    size_t start_i = 0;
    size_t end_i = current.Size();
    size_t offset = 1;
    if (rindex) {
      start_i = current.Size() - 1;
      end_i = -1;
      offset = -1;
    }
    for (size_t i = start_i; i != end_i; i += offset) {
      if (current[i].IsString()) {
        if (current[i].GetString() == value) {
          return &current[i];
        }
      } else if (current[i].IsInt()) {
        bool res = false;
        int cur_val = current[i].GetInt();
        int target_val = std::stoul(value);
        if (op == "=")
          res = cur_val == target_val;
        if (op == ">")
          res = target_val > cur_val;
        if (op == "<")
          res = target_val < cur_val;

        if (res) {
          return &current[i];
        }
      } else if (current[i].IsObject()) {
        if (check_object_(current[i], key, op, value)) {
          return &current[i];
        }
      } else {
        return nullptr;
      }
    }
    return nullptr;
  } else if (current.IsObject()) {
    if (check_object_(current, key, op, value)) {
      return &current;
    } else {
      return nullptr;
    }
  }
  return nullptr;
}

bool GJson::check_object_(Value &curr, const std::string &key,
                          const std::string &op,
                          const std::string &value) const {
  if (!curr.IsObject()) {
    return false;
  }
  if (curr.HasMember(key.c_str())) {
    if (curr[key.c_str()].IsString()) {
      if (curr[key.c_str()].GetString() == value) {
        return true;
      }
    } else if (curr[key.c_str()].IsInt()) {
      bool res = false;
      int cur_val = curr[key.c_str()].GetInt();
      int target_val = std::stoul(value);
      if (op == "=")
        res = variantToDouble(target_val) == variantToDouble(cur_val) ||
              target_val == cur_val;
      else if (op == ">")
        res = variantToDouble(target_val) > variantToDouble(cur_val) ||
              target_val > cur_val;
      else if (op == ">=")
        res = variantToDouble(target_val) >= variantToDouble(cur_val) ||
              target_val >= cur_val;
      else if (op == "<")
        res = variantToDouble(target_val) < variantToDouble(cur_val) ||
              target_val < cur_val;
      else if (op == "<=")
        res = variantToDouble(target_val) <= variantToDouble(cur_val) ||
              target_val <= cur_val;
      if (res) {
        return true;
      }
    }
  }
  return false;
}
bool GJson::update(const std::string &field, const std::string &action,
                   const std::string &val) {
  auto v = toValue(val);
  return update(field, action, v);
}

// 根据action的值来对current进行相应的修改
// +: 将Val上的值加到current上
// -: 将Val上的值减去current上
// 空白：直接替换current
// 如果有注册的通知函数，需要进行回调
bool GJson::update_(const std::string &field, const std::string &action,
                    Value &val) {
  auto l = rwlock.unique_lock();
  GLOG_DEBUG("GJson", "Updating field: " + field + " with action: " + action);

  Value *current = query_value(field);
  if (!current) {
    GLOG_DEBUG("GJson", "Field not found, attempting to create path: " + field);
    // 字段不存在，如果raw_data为object，那么设置key，value
    // 如果key是 key1;sub2;sub3;sub4这种结构，那么前面不存在的也需要进行构建
    std::vector<std::string> parts =
        split(";" + field, ';'); // 在前面新增一个保证root最开始能找到
    Value *cur_current = nullptr;
    std::string prefix = "";
    for (size_t i = 0; i < parts.size(); ++i) {
      std::string part = parts[i];
      if (prefix != "") {
        prefix = prefix + ";" + part;
      } else {
        prefix = part;
      }
      cur_current = query_value(prefix);
      if (!cur_current) {
        if (!current) {
          // 按道理来说至少有一个root，走到这里就有问题
          return false;
        }
        // 不存在，需要构建个object，然后给current
        if (i == parts.size() - 1) {
          // 添加元素
          Value cur;
          cur.CopyFrom(val, raw_data.GetAllocator());
          current->AddMember(Value(part.c_str(), raw_data.GetAllocator()), cur,
                             raw_data.GetAllocator());
        } else {
          Value obj(kObjectType);
          current->AddMember(Value(part.c_str(), raw_data.GetAllocator()), obj,
                             raw_data.GetAllocator());

          // 重新读取
          current = query_value(prefix);
          if (!current) {
            // 按道理来说至少有一个root，走到这里就有问题
            return false;
          }
        }

      } else if (cur_current->IsObject()) {
        current = cur_current;
      } else {
        // TODO 已经存在且不是object，暂时不支持修改
        return false;
      }
    }
    return true;
  }
  if (action == "+") {
    // 加法操作
    if (current->IsInt() && val.IsInt()) {
      current->SetInt(current->GetInt() + val.GetInt());
    } else if (current->IsDouble() && val.IsDouble()) {
      current->SetDouble(current->GetDouble() + val.GetDouble());
    } else if (current->IsArray()) {
      Value cur;
      cur.CopyFrom(val, raw_data.GetAllocator());
      current->PushBack(cur, raw_data.GetAllocator());
    } else {
      // 类型不匹配，无法进行加法操作
      return false;
    }
  } else if (action == "-") {
    // 减法操作
    if (current->IsInt() && val.IsInt()) {
      current->SetInt(current->GetInt() - val.GetInt());
    } else if (current->IsDouble() && val.IsDouble()) {
      current->SetDouble(current->GetDouble() - val.GetDouble());
    } else if (current->IsArray()) {
      for (Value::ValueIterator it = current->Begin(); it != current->End();) {
        if (*it == val) {
          // 找到相同的值，删除它
          it = current->Erase(it);
        } else {
          ++it;
        }
      }
    } else {
      // 类型不匹配，无法进行减法操作
      return false;
    }
  } else if (action == "~") {
    // 直接替换
    // current->CopyFrom(val, raw_data.GetAllocator());
    safeReplaceValue(current, val);
  } else {
    // 不支持的操作
    return false;
  }

  return true;
}

bool GJson::safeReplaceValue(rapidjson::Value *current,
                             const rapidjson::Value &newVal) {
  if (!current) {
    return false;
  }
  // 根据不同的类型使用不同的复制策略
  switch (newVal.GetType()) {
  case rapidjson::kNullType:
    current->SetNull();
    return true;

  case rapidjson::kFalseType:
  case rapidjson::kTrueType:
    current->SetBool(newVal.GetBool());
    return true;

  case rapidjson::kStringType:
    // 字符串类型需要使用allocator
    current->SetString(newVal.GetString(), newVal.GetStringLength(),
                       raw_data.GetAllocator());
    return true;

  case rapidjson::kNumberType:
    if (newVal.IsInt()) {
      current->SetInt(newVal.GetInt());
    } else if (newVal.IsUint()) {
      current->SetUint(newVal.GetUint());
    } else if (newVal.IsInt64()) {
      current->SetInt64(newVal.GetInt64());
    } else if (newVal.IsUint64()) {
      current->SetUint64(newVal.GetUint64());
    } else if (newVal.IsDouble()) {
      current->SetDouble(newVal.GetDouble());
    }
    return true;

  case rapidjson::kArrayType: {
    // 数组类型需要逐个复制元素
    current->SetArray();
    current->Reserve(newVal.Size(), raw_data.GetAllocator());
    for (const auto &item : newVal.GetArray()) {
      rapidjson::Value temp;
      temp.CopyFrom(item, raw_data.GetAllocator());
      current->PushBack(temp, raw_data.GetAllocator());
    }
    return true;
  }

  case rapidjson::kObjectType: {
    // 对象类型需要逐个复制成员
    current->SetObject();
    for (const auto &m : newVal.GetObject()) {
      rapidjson::Value key;
      key.CopyFrom(m.name, raw_data.GetAllocator());

      rapidjson::Value value;
      value.CopyFrom(m.value, raw_data.GetAllocator());

      current->AddMember(key, value, raw_data.GetAllocator());
    }
    return true;
  }

  default:
    return false;
  }
}

void GJson::trigger_callbacks(const std::string &field) {
  std::vector<std::string> parts = split(field, ';');

  // 构建完整路径并收集回调
  TrieNode *current = callback_trie_.get();
  if (current == nullptr) {
    return;
  }
  std::string current_path;
  for (size_t i = 0; i < parts.size(); ++i) {
    if (!current_path.empty()) {
      current_path += ";";
    }
    current_path += parts[i];

    if (current->children.count(parts[i])) {
      current = current->children[parts[i]].get();
    } else {
      break;
    }
  }
  if (current_path != field) {
    return;
  }

  std::vector<std::tuple<std::string, CallbackFunc, int>> callbacks_to_trigger;
  collect_affected_callbacks(current, current_path, callbacks_to_trigger);

  // 执行收集到的回调
  std::unordered_set<std::string> null_paths; // 避免重复调用
  for (const auto &[callback_path, callback, idx] : callbacks_to_trigger) {
    if (null_paths.count(callback_path) > 0) {
      continue;
    }
    // 获取回调路径对的值
    const rapidjson::Value *callback_value = query_value(callback_path);
    if (callback_value == nullptr) {
      null_paths.insert(callback_path);
      continue;
    }
    if (!callback(callback_path, callback_value)) {
      // 返回false，说明这一个callback已经回收了，那么这里也需要清理掉
      remove_callback_item(callback_trie_.get(), callback_path, idx);
    }
  }
}

// 递归寻找字典树，寻找给定路径下的所有子节点isend为true的，然后将其回调加入回调队列
// 获取所有需要触发的回调
void GJson::collect_affected_callbacks(
    TrieNode *node, const std::string &base_path,
    std::vector<std::tuple<std::string, CallbackFunc, int>> &callbacks) {
  // 如果当前节点是终点，添加回调
  if (node->is_endpoint) {
    int idx = 0;
    for (const auto &callback : node->callbacks) {
      callbacks.emplace_back(base_path, callback, idx++);
    }
  }

  // 递归处理所有子节点
  for (const auto &child : node->children) {
    std::string new_path = base_path;
    if (!new_path.empty()) {
      new_path += ";";
    }
    new_path += child.first;
    collect_affected_callbacks(child.second.get(), new_path, callbacks);
  }
}

// 删除callback字典树中的callback列表中指定一个元素的位置
void GJson::remove_callback_item(TrieNode *node, const std::string &base_path,
                                 int idx) {
  if (node == nullptr) {
    return;
  }
  std::istringstream iss(base_path);
  std::string token;
  while (std::getline(iss, token, ';')) {
    if (token.empty()) {
      continue;
    }
    if (node->children.find(token) == node->children.end()) {
      return;
    }
    node = node->children[token].get();
  }
  if (idx >= 0 && idx < node->callbacks.size()) {
    node->callbacks.erase(node->callbacks.begin() + idx);
  }
}

void GJson::trigger_all_callbacks() {
  // 遍历callback_trie_树节点，并且保留路径上的名字，如果is_endpoint为true，获取整个路径名用于数据查询，将这个数据传给callbacks上
  std::vector<std::tuple<std::string, CallbackFunc, int>> callbacks_to_trigger;
  collect_affected_callbacks(callback_trie_.get(), "", callbacks_to_trigger);
  for (const auto &[callback_path, callback, idx] : callbacks_to_trigger) {
    const rapidjson::Value *callback_value = query_value(callback_path);
    callback(callback_path, callback_value);
  }
}

void GJson::load_or_store(const std::string &data) {
  if (data.empty()) {
    GLOG_WARNING("GJson", "Attempted to load empty data");
    return;
  }

  auto write = rwlock.unique_lock();
  GLOG_DEBUG("GJson", "Parsing JSON data");
  raw_data.Parse(data.c_str());

  if (raw_data.HasParseError()) {
    GLOG_ERROR("GJson", "JSON parse error at offset: " +
                            std::to_string(raw_data.GetErrorOffset()));
    return;
  }

  if (imported_) {
    GLOG_DEBUG("GJson", "Triggering callbacks for data update");
    trigger_all_callbacks();
  }
  imported_ = true;
}

} // namespace gamedb