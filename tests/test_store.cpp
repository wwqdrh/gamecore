#include <gtest/gtest.h>
#include <iostream>

#include "store.h"

using namespace gamedb;

TEST(STORETest, TestEncrypt) {
  FileStore file("test_store_encrypt.txt");
  file.saveData("Hello, world!");
  std::string data = file.loadData();
  EXPECT_EQ(data, "Hello, world!");
  // 删除test.txt文件
  std::remove("test_store_encrypt.txt");
}

TEST(STORETest, TestCustomFunc) {
  std::string buff = "";
  FileStore file([&buff]() { return buff; },        // 自定义读取函数
                 [&buff](const std::string &data) { // 自定义保存函数
                   buff = data;
                 });
  file.saveData("HelloWorld");
  ASSERT_TRUE(buff != "" && buff != "HelloWorld");
  ASSERT_EQ(file.loadData(), "HelloWorld");
}