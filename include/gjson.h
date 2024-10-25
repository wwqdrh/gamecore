#pragma once

#include <algorithm>
#include <random>
#include <sstream>
#include <stdexcept>
#include <vector>

#include "rapidjson/document.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/writer.h"

using namespace rapidjson;

namespace gamedb {
class GJson {
private:
  Document raw_data;

private:
  std::vector<std::string> split(const std::string &s, char delimiter) const;

  Value *traverse(Value &current, const std::string &key);

  Value getRandomElements(Value &current, size_t count);
  Value *getCompareElements(Value &current, const std::string &key,
                            const std::string &op, const std::string &value,
                            bool rindex = false);

  Value *query_value(const std::string &field);

public:
  explicit GJson(const std::string &data) { raw_data.Parse(data.c_str()); }
  void parse_file(const std::string &filename);
  Value parse(const std::string &data);
  std::string query(const std::string &field); // 返回的是json字符串
  std::vector<std::string> keys(const std::string &field);
  std::vector<std::string> values(const std::string &field);
  bool update(const std::string &field, const std::string &action, Value &val);
};
} // namespace libs