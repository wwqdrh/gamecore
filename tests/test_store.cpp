#include <_types/_uint8_t.h>
#include <gtest/gtest.h>
#include <iostream>
#include <string>
#include <vector>

#include "store.h"

using namespace gamedb;

TEST(STORETest, TestEncrypt) {
  FileStore file("test_store_encrypt.txt");
  file.saveData("Hello, world!");
  std::string data = file.loadData();
  EXPECT_EQ(data, "Hello, world!");

  // 测试json字符串格式
  file.saveData(R"({"name": "gamedb"})");
  data = file.loadData();
  EXPECT_EQ(data, R"({"name": "gamedb"})");

  // 删除test.txt文件
  std::remove("test_store_encrypt.txt");
}

TEST(STORETest, TestCustomFunc) {
  std::vector<uint8_t> buff;
  FileStore file([&buff]() { return buff; },          // 自定义读取函数
                 [&buff](std::vector<uint8_t> data) { // 自定义保存函数
                   buff = data;
                 });
  file.saveData("HelloWorld");
  ASSERT_EQ(file.loadData(), "HelloWorld");
}