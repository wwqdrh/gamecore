#include <gtest/gtest.h>

#include "libgjson.h" // 假设 GJson 类的声明在这个头文件中

using namespace libs;

TEST(GJsonTest, ParseAndQuery) {
  // 创建一个 GJson 对象
  GJson json(R"({
        "name": "John Doe",
        "age": 30,
        "city": "New York",
        "is_student": false,
        "grades": [85, 90, 78],
        "address": {
            "street": "123 Main St",
            "zip": "10001"
        }
    })");

  // 使用 query 方法测试各种数据类型的获取
  EXPECT_EQ(json.query("name"), "\"John Doe\"");
  EXPECT_EQ(json.query("age"), "30");
  EXPECT_EQ(json.query("city"), "\"New York\"");
  EXPECT_EQ(json.query("is_student"), "false");
  EXPECT_EQ(json.query("grades;0"), "85");
  EXPECT_EQ(json.query("address;street"), "\"123 Main St\"");
}