#include <fstream>
#include <iostream>
#include <mutex>
#include <shared_mutex>
#include <sstream>
#if defined(_WIN32) || defined(_WIN64)
#include <numeric>
#endif

#include "gjson.h"
#include "rapidjson/document.h"
#include "rapidjson/rapidjson.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"

using namespace rapidjson;

namespace gamedb {

// =============
// public
// =============
Value *GJson::query_value(const std::string &field) const {
  // std::shared_lock<std::shared_mutex> lock(rw_mtx);
  std::lock_guard<std::recursive_mutex> lock(mutex_);

  std::vector<std::string> parts = split(field, ';');
  // Value curr_data;
  // curr_data.CopyFrom(raw_data, raw_data.GetAllocator());
  // Value *current = &raw_data;
  Value *current = const_cast<Value *>(static_cast<const Value *>(&raw_data));

  for (const auto &part : parts) {
    if (current == nullptr) {
      break;
    }
    if (part.empty())
      continue;

    if (part[0] == '#') {
      // Special operation
      // example1: #(a=1)
      // example2: #(@=name), 对于object来说，寻找key为name所对应的值
      // example3: #last('age' | '', '>', 10),
      // 对于数组来说，如果第一个参数有值那么就是针对数组[字典]，寻找字典中满足大于10的最后一个元素，否组就是当前数组中的大于10的最后一个元素
      // example3: #first('age' | '', '>', 10), 同上
      // example3: #random
      if (part[1] == '(' && part.back() == ')') {
        // Case 3: Condition within parentheses
        std::string condition = part.substr(2, part.length() - 1 - 2);
        std::cout << condition << std::endl;
        std::vector<std::string> condParts = split(condition, '=');
        if (condParts.size() != 2) {
          // std::cerr << "Invalid condition format: " << condition <<
          // std::endl;
          continue;
        }

        if (current->IsObject() && condParts[0] == "@") {
          for (auto it = current->MemberBegin(); it != current->MemberEnd();
               ++it) {
            if (it->name.IsString() && it->name.GetString() == condParts[1]) {
              current = &it->value;
              break;
            }
          }
        }
      } else {
        // Special operation
        // 将part字符串()中的提取出来，并且使用,分割，并且获取#last()括号之前的字符串
        std::string operation = part.substr(1, part.find('(') - 1);
        std::vector<std::string> ops =
            split(part.substr(part.find('(') + 1, part.length() - 2), ',');
        if (operation == "last") {
          current = getCompareElements(*current, ops[0], ops[1], ops[2], true);
          continue;
        } else if (operation == "first") {
          current = getCompareElements(*current, ops[0], ops[1], ops[2], false);
          continue;
        } else if (operation == "random") {
          size_t count = std::stoul(ops[0]);
          Value randomVal = getRandomElements(*current, count);
          current = &randomVal;
        }
      }
    } else {
      // Normal key or index
      Value *next = traverse(*current, part);
      if (next == nullptr) {
        return next;
      }
      current = next;
    }
  }
  // std::cout << current->Size() << std::endl;
  return current;
}

// 解析传入的查询参数
// 首先使用;符号分割成查询链条
// 1、每一个部分，如果不以#开头，则就是简单的key参数，如果是数字就是对应的位置的元素
// 2、每一个部分，如果以#开头，那么就是特殊操作，例如#random:10，就是指在当前的object(需要适配{}和[])中随机获取10个元素
// 3、如果是#开头然后包含一个括号(),
// 那么括号里面的就是一个解析参数，例如#(@=1)就是判断当前的object的key==1,
// 注意这里都是字符串
std::string GJson::query(const std::string &field) const {
  // std::shared_lock<std::shared_mutex> lock(rw_mtx);
  std::lock_guard<std::recursive_mutex> lock(mutex_);

  Value *current = query_value(field);
  if (current == nullptr) {
    return "";
  }
  // std::cout << current->Size() << std::endl;
  StringBuffer buffer;
  Writer<StringBuffer> writer(buffer);
  current->Accept(writer);
  return buffer.GetString();
}

std::vector<std::string> GJson::keys(const std::string &field) const {
  // std::shared_lock<std::shared_mutex> lock(rw_mtx);
  std::lock_guard<std::recursive_mutex> lock(mutex_);

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
  // std::shared_lock<std::shared_mutex> lock(rw_mtx);
  std::lock_guard<std::recursive_mutex> lock(mutex_);

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
  if (current.IsObject()) {
    if (current.HasMember(key.c_str())) {
      return &current[key.c_str()];
    }
  } else if (current.IsArray()) {
    size_t index = std::stoul(key);
    if (index < current.Size()) {
      return &current[index];
    }
  }
  return nullptr;
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
        if (current[i].HasMember(key.c_str())) {
          if (current[i][key.c_str()].IsString()) {
            if (current[i][key.c_str()].GetString() == value) {
              return &current[i];
            }
          } else if (current[i][key.c_str()].IsInt()) {
            bool res = false;
            int cur_val = current[i][key.c_str()].GetInt();
            int target_val = std::stoul(value);
            if (op == "=")
              res = target_val == cur_val;
            if (op == ">")
              res = target_val > cur_val;
            if (op == "<")
              res = target_val < cur_val;
            if (res) {
              return &current[i];
            }
          }
        }
      } else {
        return nullptr;
      }
    }
    return nullptr;
  }
  return nullptr;
}

void GJson::parse_file(const std::string &filename) {
  // std::unique_lock<std::shared_mutex> lock(rw_mtx);
  std::lock_guard<std::recursive_mutex> lock(mutex_);

  std::ifstream file(filename);
  if (!file.is_open()) {
    raw_data.Parse("{}");
    return;
  }
  std::stringstream buffer;
  buffer << file.rdbuf();
  raw_data.Parse(buffer.str().c_str());
  file.close();
}

Value GJson::parse(const std::string &data) {
  std::lock_guard<std::recursive_mutex> lock(mutex_);

  rapidjson::Document doc;
  doc.Parse(data.c_str());
  if (doc.HasParseError()) {
    return Value();
  }
  rapidjson::Value val;
  val.CopyFrom(doc, doc.GetAllocator());
  return val;
}
// 根据action的值来对current进行相应的修改
// +: 将Val上的值加到current上
// -: 将Val上的值减去current上
// 空白：直接替换current
bool GJson::update_(const std::string &field, const std::string &action,
                    Value &val) {
  std::lock_guard<std::recursive_mutex> lock(mutex_);
  Value *current = query_value(field);
  if (!current) {
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
        // 不存在，需要构建一个object，然后给current
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
    current->CopyFrom(val, raw_data.GetAllocator());
  } else {
    // 不支持的操作
    return false;
  }

  return true;
}
} // namespace gamedb