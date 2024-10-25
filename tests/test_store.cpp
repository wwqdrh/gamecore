#include <iostream>
#include <gtest/gtest.h>

#include "store.h"

using namespace gamedb;

TEST(STORETest, TestEncrypt) {
    FileManager file("test_store_encrypt.txt");
    file.saveData("Hello, world!");
    std::string data = file.loadData();
    EXPECT_EQ(data, "Hello, world!");
    // 删除test.txt文件
    std::remove("test_store_encrypt.txt");
}