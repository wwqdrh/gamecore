#include <gtest/gtest.h>
#include <iostream>
#include <vector>

#include "question/manager.h"

using namespace gamedb;

TEST(QuestionTest, TestBasic) {
  QuesManager ques;
  std::vector<int> ava_ids;
  std::vector<int> active_ids;

  ASSERT_TRUE(ques.addTask(1));
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 1);
  ASSERT_FALSE(ques.addTask(1));  // 不能重复添加
  ASSERT_TRUE(ques.startTask(1)); // 开始任务
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 0);
  active_ids = ques.get_active_task();
  ASSERT_EQ(active_ids.size(), 1);
  ASSERT_FALSE(ques.startTask(2));    // 不存在的任务不能开始
  ASSERT_TRUE(ques.completeTask(1));  // 完成任务
  ASSERT_FALSE(ques.completeTask(2)); //
}

TEST(QuestionTest, TestLoadFile) {
  std::string test_file = "test_question_loadfile.json";

  auto json = std::make_shared<GJson>(std::make_shared<FileStore>(test_file));

  QuesManager ques;
  ques.set_store(json);

  std::vector<int> ava_ids;
  std::vector<int> active_ids;
    std::cout << json->query("") << std::endl;
  ASSERT_TRUE(ques.addTask(1));
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 1);
  ASSERT_FALSE(ques.addTask(1));  // 不能重复添加
  ASSERT_TRUE(ques.startTask(1)); // 开始任务
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 0);
  active_ids = ques.get_active_task();
  ASSERT_EQ(active_ids.size(), 1);
  ASSERT_FALSE(ques.startTask(2));    // 不存在的任务不能开始
  ASSERT_TRUE(ques.completeTask(1));  // 完成任务
  ASSERT_FALSE(ques.completeTask(2)); //

  // 删除test_filesave.json文件
  std::remove(test_file.c_str());
}
