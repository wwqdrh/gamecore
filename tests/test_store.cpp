#include <iostream>
#include <gtest/gtest.h>

#include "store.h"

TEST(STORETest, TestEncrypt) {
    libs::FileManager file("test_store_encrypt.txt");
    file.saveData("Hello, world!");
    std::string data = file.loadData();
    EXPECT_EQ(data, "Hello, world!");
    // 删除test.txt文件
    std::remove("test_store_encrypt.txt");
}